pub mod arguments;
mod check;
mod errors;
mod repo;
mod run;
mod world;

pub use arguments::parse_args;
pub use run::run;
pub use world::World;

// #[derive(Debug)]
// struct BlobbedFile {
//     pub filename: PathBuf,
//     pub contents: Vec<u8>,
// }

// impl BlobbedFile {
//     pub fn new(repo: &Repository, delta: DiffDelta<'_>) -> Result<Self> {
//         let blob = repo.find_blob(delta.new_file().id())?;
//         Ok(Self {
//             filename: delta
//                 .old_file()
//                 .path()
//                 .ok_or_else(|| anyhow!("Invalid path"))?
//                 .into(),
//             contents: blob.content().to_owned(),
//         })
//     }
// }

// async fn validate_file_status(
//     semaphore: &Semaphore,
//     file: &BlobbedFile,
//     command: OsString,
// ) -> Result<bool> {
//     let result = {
//         let _guard = semaphore.acquire().await;
//         let mut child = Command::new("sh")
//             .arg("-c")
//             .arg(command)
//             .stdin(Stdio::piped())
//             .stdout(Stdio::inherit())
//             .stderr(Stdio::inherit())
//             .spawn()?;

//         let mut stdin = child.stdin.take().expect("stdin is not a pipe");
//         let (_, result) = join!(stdin.write_all(&file.contents), child.status());
//         result?
//     };

//     Ok(result.success())
// }

// async fn validate_file(semaphore: &Semaphore, file: BlobbedFile, arguments: &Args) -> Result<()> {
//     let mut futures = FuturesUnordered::new();
//     for validation in &arguments.validate_commands {
//         match validation {
//             arguments::Mode::Status(command) => {
//                 let command = command
//                     .as_encoded_bytes()
//                     .replace(
//                         arguments.placeholder.as_encoded_bytes(),
//                         file.filename.as_os_str().as_encoded_bytes(),
//                     )
//                     .into_os_string()?;
//                 futures.push(validate_file_status(semaphore, &file, command))
//             }
//             _ => panic!("not supported!"),
//         }
//     }

//     while let Some(result) = futures.next().await {
//         dbg!(result)?;
//     }
//     Ok(())
// }
