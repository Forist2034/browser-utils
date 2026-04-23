use std::{
    fs::{self, File},
    io::{Read, Seek, Stdin, Stdout, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};

use anyhow::Context;
use chrono::{DateTime, Datelike};
use serde::Deserialize;
use uuid::Uuid;

use browser_utils_history_core::{Entry, EntryTitle, Event, EventKind, Info};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Message {
    Init {
        root: String,
    },
    OnVisit {
        id: String,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        title: Option<String>,
    },
    OnTitleUpdate {
        id: String,
        #[serde(default)]
        title: Option<String>,
    },
    Disconnect,
}

fn handle_message(
    event_file: &mut File,
    entries: &mut indexmap::IndexMap<String, Entry<String>>,
    event_buf: &mut Vec<u8>,
    msg: Message,
) -> anyhow::Result<()> {
    let timestamp: DateTime<chrono::FixedOffset> = chrono::Local::now().into();
    let id = Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        timestamp.timestamp() as u64,
        timestamp.timestamp_subsec_nanos(),
    ));

    event_buf.clear();
    event_buf.push(0x1e);
    match msg {
        Message::Init { .. } => anyhow::bail!("repeated init message"),
        Message::OnVisit {
            id: browser_id,
            url,
            title,
        } => {
            serde_json::to_writer(
                &mut *event_buf,
                &Event {
                    id,
                    browser_id: &browser_id,
                    timestamp,
                    event: EventKind::Visit {
                        url: url.as_ref().map(String::as_str),
                        title: title.as_ref().map(String::as_str),
                    },
                },
            )
            .unwrap();

            entries.insert(
                browser_id.clone(),
                Entry {
                    id,
                    visit_event_id: id,
                    timestamp,
                    url,
                    titles: Vec::from([EntryTitle {
                        event_id: id,
                        timestamp,
                        title,
                    }]),
                },
            );
        }
        Message::OnTitleUpdate {
            id: browser_id,
            title,
        } => {
            let entry = entries
                .get_mut(&browser_id)
                .context("missing visit event")?;
            serde_json::to_writer(
                &mut *event_buf,
                &Event {
                    id,
                    browser_id: &browser_id,
                    timestamp,
                    event: EventKind::TitleUpdate {
                        visit_event: entry.visit_event_id,
                        title: title.as_ref().map(String::as_str),
                    },
                },
            )
            .unwrap();

            entry.titles.push(EntryTitle {
                event_id: id,
                timestamp,
                title,
            });
        }
        Message::Disconnect => return Ok(()),
    };

    event_buf.push(b'\n');
    event_file
        .write_all(&event_buf)
        .context("failed to write event file")
}

fn read_message(buf: &mut Vec<u8>, input: &mut Stdin) -> anyhow::Result<Option<Message>> {
    let mut len_buf = [0; 4];
    match input.read_exact(&mut len_buf).and_then(|_| {
        buf.resize(u32::from_ne_bytes(len_buf) as usize, 0);
        input.read_exact(buf)
    }) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(anyhow::Error::new(e).context("failed to read input")),
    }

    serde_json::from_slice(buf.as_slice())
        .map(Some)
        .context("failed to deserialize message")
}
fn reply_error(e: &anyhow::Error, output: &mut Stdout) -> anyhow::Result<()> {
    let msg = format!("{e:?}");
    eprintln!("{msg}");

    let mut buf = Vec::from([0; size_of::<u32>()]);
    serde_json::to_writer(&mut buf, &msg).unwrap();
    *buf.first_chunk_mut().unwrap() = ((buf.len() - size_of::<u32>()) as u32).to_ne_bytes();
    output.write_all(&buf).context("failed to write reply")
}

fn wait_root(input: &mut Stdin, output: &mut Stdout) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    loop {
        match read_message(&mut buf, &mut *input).context("failed to read input message")? {
            Some(Message::Init { root }) => return Ok(root),
            Some(Message::Disconnect) => std::process::exit(0),
            Some(_) => reply_error(
                &anyhow::Error::msg("storage root should be sent first"),
                &mut *output,
            )?,
            None => std::process::exit(0),
        }
    }
}

fn run(input: &mut Stdin, output: &mut Stdout) -> anyhow::Result<()> {
    let start_time: DateTime<chrono::FixedOffset> = chrono::Local::now().into();
    let id = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        start_time.timestamp() as u64,
        start_time.timestamp_subsec_nanos(),
    ));

    let root = wait_root(&mut *input, &mut *output)?;

    let mut base_path = format!(
        "{root}/{year}/{year}-{month:02}",
        year = start_time.year(),
        month = start_time.month()
    );
    fs::create_dir_all(&base_path).context("failed to create dir")?;
    {
        use std::fmt::Write;
        let _ = write!(&mut base_path, "/{id}");
    }
    let base_path = Path::new(&base_path);

    let hostname =
        fs::read_to_string("/proc/sys/kernel/hostname").context("failed to get hostname")?;
    let hostname = hostname.trim();

    let mut info_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o444)
        .open(base_path.with_added_extension("info.json"))
        .context("failed to create info file")?;
    let mut event_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o444)
        .open(base_path.with_added_extension("events.json"))
        .context("failed to create event file")?;
    let mut entries_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o444)
        .open(base_path.with_added_extension("entries.json"))
        .context("failed to create entry json file")?;

    info_file
        .write_all(
            &serde_json::to_vec_pretty(&Info {
                id,
                hostname,
                start_time,
                end_time: None,
            })
            .unwrap(),
        )
        .context("failed to write info")?;

    let mut in_buf = Vec::new();
    let mut event_buf = Vec::new();
    let mut entries = indexmap::IndexMap::new();

    while let Some(msg) = read_message(&mut in_buf, &mut *input)? {
        if let Err(e) = handle_message(&mut event_file, &mut entries, &mut event_buf, msg) {
            reply_error(&e, &mut *output)?;
        }
    }

    event_buf.clear();
    serde_json::to_writer_pretty(&mut event_buf, &entries.into_values().collect::<Vec<_>>())
        .unwrap();
    entries_file
        .write_all(&mut event_buf)
        .context("failed to write entries")?;

    event_buf.clear();
    serde_json::to_writer_pretty(
        &mut event_buf,
        &Info {
            id,
            hostname,
            start_time,
            end_time: Some(chrono::Local::now().into()),
        },
    )
    .unwrap();
    info_file
        .seek(std::io::SeekFrom::Start(0))
        .and_then(|_| {
            info_file.write_all(&event_buf)?;
            info_file.set_len(event_buf.len() as u64)
        })
        .context("failed to update info file")
}

fn main() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let ret = run(&mut stdin, &mut stdout);
    if let Err(e) = &ret {
        let _ = reply_error(e, &mut stdout);
    }
    ret
}
