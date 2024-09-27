use notify::{Config, PollWatcher, RecursiveMode, Watcher};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

struct FileTail {
    file: std::fs::File,
    offset: u64,
}

impl FileTail {
    fn new(path: &str) -> std::io::Result<FileTail> {
        let mut file = OpenOptions::new().read(true).write(false).open(path)?;
        let offset = file.seek(SeekFrom::End(0))?;
        print!("{}", read_last_n_lines(path, 100).unwrap());
        Ok(FileTail { file, offset })
    }

    fn update(&mut self) -> std::io::Result<()> {
        let pos = self.file.seek(SeekFrom::End(0))?;
        if pos > self.offset {
            self.file.seek(SeekFrom::Start(self.offset))?;
            self.offset = pos;
            let mut buf = Vec::new();
            self.file.read_to_end(&mut buf)?;
            print!("{}", String::from_utf8_lossy(&buf));
        }
        Ok(())
    }
}
fn read_last_n_lines(path: &str, n: usize) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut file_size = file.seek(SeekFrom::End(0))?;

    // 确保文件至少有n+1行，否则从头读取
    if file_size <= 1 {
        return Ok(String::new());
    }

    let mut lines_found = 0;
    let mut pos = file_size;
    let mut buffer = Vec::new();

    // 从文件末尾开始寻找换行符
    loop {
        file.seek(SeekFrom::End(-(pos as i64)))?;
        let mut tmp = [0u8];
        file.read_exact(&mut tmp)?;
        if tmp[0] == b'\n' {
            lines_found += 1;
            if lines_found == n + 1 {
                break;
            }
        }
        pos -= 1;
        if pos == 0 {
            break;
        }
    }

    // 如果找到了足够的行数，从找到的位置读取数据xxx
    if lines_found > 0 {
        file.seek(SeekFrom::End(-(pos as i64)))?;
        file.read_to_end(&mut buffer)?;
    } else {
        file.seek(SeekFrom::Start(0))?;
        file.read_to_end(&mut buffer)?;
    }

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: PollWatcher = Watcher::new(tx, Config::default()
        .with_compare_contents(true)
        .with_poll_interval(Duration::from_millis(10)))?;
    let path = "logs\\sys-console.log";
    watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
    let mut tail = FileTail::new(path)?;

    loop {
        while let Ok(event) = rx.recv() {
            let _ = tail.update();
        }
    }
}