use std::{
    io::{self, BufRead, BufReader},
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

impl UprojectFile {
    pub fn from_path(path: &Path) -> Result<Self, io::Error> {
        let file = std::fs::read(path)?;

        let result = serde_json::from_slice::<Self>(&file);

        match result {
            Ok(uproject) => Ok(uproject),
            Err(err) => Err(io::Error::other(err)),
        }
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

    pub fn from_cwd(search_options: SearchOptions) -> Result<Self, UnrealError> {
        let cwd = std::env::current_dir()?;

        if let Ok(project) = Self::load_cached_project(&cwd) {
            return Ok(project);
        }

        let uproject_path = search::file_with_extension(&cwd, "uproject")
            .map_err(|_| UnrealError::ProjectNotFound)?;

        let uproject = UprojectFile::from_path(&uproject_path)?;

        let name = Self::extract_project_name(&uproject_path)?;
        let version = Self::parse_version(&uproject.unreal_version)?;

        let unreal_installation = UnrealInstallation::find_installation(version, search_options)?;

        let project = UnrealProject {
            unreal_version: version,
            name,
            associated_engine: unreal_installation,
            path: uproject_path.to_string_lossy().to_string(),
        };

        let _ = Self::save_project(&project);
        Ok(project)
    }

    /* Attempts to save the UnrealProject to a unreal.project file. */
    fn save_project(project: &UnrealProject) -> io::Result<()> {
        let file = std::fs::File::create("unreal.project")?;

        let result = match serde_json::to_writer(file, &project) {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::other(err.to_string())),
        };

        if result.is_ok() {
            println!("Saved Project.");
        }
        result
    }

    /* Will attempt to load an UnrealProject from a .project file in the current directory, returns None if it fails. */
    fn load_cached_project(cwd: impl AsRef<Path>) -> io::Result<UnrealProject> {
        let path = search::file_with_extension(cwd, "project")?;

        let handle = std::fs::File::open(&path)?;
        let project = serde_json::from_reader(&handle)?;

        Ok(project)
    }

    pub fn build_project(&self) -> Result<(), UnrealError> {
        let ue_path = &self.associated_engine.base_path;

        let build_bat = format!("{ue_path}\\Engine\\Build\\BatchFiles\\Build.bat");

        let args = &[
            "/C",
            "call",
            &build_bat,
            &format!("{}Editor", self.name),
            "Win64",
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
