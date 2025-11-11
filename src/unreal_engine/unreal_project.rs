use std::{
    borrow::Borrow,
    io::{self, BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::{
    unreal_engine::{error::UnrealError, unreal_installation::UnrealInstallation},
    utility::{
        path_utility::{self},
        search::{self, SearchOptions},
    },
};

#[derive(Serialize, Deserialize)]
pub struct UnrealProject {
    unreal_version: f32,
    name: String,
    associated_engine: UnrealInstallation,
    path: String,
}

#[derive(Deserialize)]
struct UprojectFile {
    #[serde(alias = "EngineAssociation")]
    unreal_version: String,
}

impl TryFrom<&Path> for UprojectFile {
    type Error = std::io::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let file = std::fs::read(value)?;

        let result = serde_json::from_slice::<Self>(&file);

        match result {
            Ok(uproject) => Ok(uproject),
            Err(err) => Err(io::Error::other(err)),
        }
    }
}

impl TryFrom<SearchOptions> for UnrealProject {
    type Error = UnrealError;

    fn try_from(value: SearchOptions) -> Result<Self, Self::Error> {
        let cwd = &value.project_directory;

        if let Ok(project) = Self::load_cached_project(cwd) {
            return Ok(project);
        }

        let uproject_path =
            search::file_with_extension(cwd, "uproject").ok_or(UnrealError::ProjectNotFound)?;

        let uproject = UprojectFile::try_from(uproject_path.as_path())?;

        let name = Self::extract_project_name(&uproject_path)?;
        let version = Self::parse_version(&uproject.unreal_version)?;

        let unreal_installation = UnrealInstallation::try_from(value)?;
        println!("{unreal_installation:?}");

        let project = UnrealProject {
            unreal_version: version,
            name,
            associated_engine: unreal_installation,
            path: uproject_path.to_string_lossy().to_string(),
        };

        let _ = Self::save_project(&project);
        Ok(project)
    }
}

impl UnrealProject {
    fn extract_project_name(uproject_path: &Path) -> Result<String, UnrealError> {
        let name = path_utility::remove_extension(uproject_path);
        path_utility::filename_as_string(&name).ok_or(UnrealError::ProjectNotFound)
    }

    fn parse_version(version_str: &str) -> Result<f32, UnrealError> {
        version_str.parse().or(Err(UnrealError::ProjectNotFound))
    }

    /* Attempts to save the UnrealProject to a unreal.project file. */
    fn save_project(project: &UnrealProject) -> io::Result<()> {
        let mut file = std::fs::File::create("unreal.project")?;

        let toml_str = match toml::to_string_pretty(project) {
            Ok(str) => Ok(str),
            Err(err) => Err(std::io::Error::other(err.to_string())),
        };

        let result = toml_str.and_then(|str| write!(&mut file, "{}", &str));

        if result.is_ok() {
            println!("Saved Project.");
        }

        result.map_err(|err| io::Error::other(err.to_string()))
    }

    /* Will attempt to load an UnrealProject from a .project file in the current directory, returns None if it fails. */
    fn load_cached_project(cwd: impl AsRef<Path>) -> io::Result<UnrealProject> {
        let path = search::file_with_extension(cwd, "project")
            .ok_or(io::Error::from(ErrorKind::NotFound))?;

        let handle = std::fs::File::open(&path)?;
        let project = serde_json::from_reader(&handle)?;

        Ok(project)
    }

    pub fn build_project(&self) -> Result<(), UnrealError> {
        let file = self
            .associated_engine
            .find_file("Build.bat")
            .ok_or(UnrealError::EngineNotFound)?;

        let build_bat = file.to_str().unwrap();

        let args = &[
            "/C",
            "call",
            build_bat,
            &format!("{}Editor", self.name),
            #[cfg(target_os = "windows")]
            "Win64",
            #[cfg(not(target_os = "windows"))]
            "Linux",
            "Development",
            &self.path,
            "-waitmutex",
            "-NoHotReload",
        ];

        println!("\nStarting Project Compilation..");
        let mut child = create_child_process(args);

        monitor_output(&mut child);

        let status = match child.wait() {
            Ok(status) => status,
            Err(err) => return Err(UnrealError::IoError { error: err }),
        };

        if !status.success() {
            return Err(UnrealError::FailedExitStatus { status });
        }

        Ok(())
    }

    pub fn build_and_start(self) -> Result<(), UnrealError> {
        self.build_project().and_then(|_| self.start_project())
    }

    pub fn start_project(self) -> Result<(), UnrealError> {
        let args = &["/C", "start", &self.associated_engine.exe_path, &self.path];

        println!("\nStarting Project..");
        let mut child = create_child_process(args);

        monitor_output(&mut child);

        let status = match child.wait() {
            Ok(status) => status,
            Err(err) => return Err(UnrealError::IoError { error: err }),
        };

        if !status.success() {
            return Err(UnrealError::FailedExitStatus { status });
        }

        Ok(())
    }
}

#[inline]
fn create_child_process(args: &[&str]) -> Child {
    Command::new("cmd")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start build command.")
}

fn monitor_output(child: &mut Child) {
    let stdout = BufReader::new(child.stdout.take().expect("No stdout"));
    let stderr = BufReader::new(child.stderr.take().expect("No stderr"));

    let out_lines = stdout.lines();
    let err_lines = stderr.lines();

    let barrier = std::sync::Barrier::new(2);

    rayon::scope(|s| {
        s.spawn(|_| {
            for line in out_lines.into_iter().map_while(Result::ok) {
                println!("{line}");
            }

            barrier.wait();
        });

        for line in err_lines.into_iter().map_while(Result::ok) {
            eprintln!("{line}");
        }

        barrier.wait();
    });
}
