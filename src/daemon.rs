use std::env;
use std::process::Command;

use sqlx::SqlitePool;

pub fn get_daemon() -> String {
    let mut daemon_str;

    let exedir = env::current_exe().unwrap().to_str().unwrap().to_string();

    if cfg!(target_os = "linux") {
        daemon_str = format!(
            r#"


            [Unit]
Description=DocSearch Daemon
After=network.target

[Service]
Type=simple

ExecStart={} begin

Restart=always
RestartSec=3

StandardOutput=append:/tmp/docsearch_new.out
StandardError=append:/tmp/docsearch_new.err

[Install]
WantedBy=multi-user.target

            "#,
            exedir
        )
    } else if cfg!(target_os = "macos") {
        daemon_str = format!(
            r#"

                 <?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">

<plist version="1.0">
<dict>

    <key>Label</key>
    <string>com.jeeva.docsearch</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>begin</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/docsearch_new.out</string>

    <key>StandardErrorPath</key>
    <string>/tmp/docsearch_new.err</string>

</dict>
</plist>"#,
            exedir
        );
    } else {
        daemon_str = "".to_string();
    }

    return daemon_str;
}

pub fn start_daemon(daemonpath: &str) {
    if cfg!(target_os = "macos") {
        let output = Command::new("id").arg("-u").output().unwrap();

        let uid = String::from_utf8(output.stdout).unwrap().trim().to_string();

        match Command::new("launchctl")
            .args(["bootstrap", &format!("gui/{}", uid), daemonpath])
            .status()
        {
            Ok(_) => {
                println!("Daemon Started Successfully");
            }

            Err(_) => {
                println!("Error starting the daemon");
                return;
            }
        }
    } else if cfg!(target_os = "linux") {
        let status = Command::new("systemctl")
            .args(["--user", "link", daemonpath])
            .status()
            .expect("Failed to run systemctl link");

        // systemctl --user enable --now main.service
        let status = Command::new("systemctl")
            .args(["--user", "enable", "--now", "docsearch.service"])
            .status()
            .expect("Failed to enable service");

        println!("Daemon Started Successfully");
    } else {
        println!("No Support for Windows. Get a better OS!!!");
    }
}

pub fn stop_daemon(daemonpath: &str) {
    if cfg!(target_os = "macos") {
        let output = Command::new("id").arg("-u").output().unwrap();

        let uid = String::from_utf8(output.stdout).unwrap().trim().to_string();

        match Command::new("launchctl")
            .args(["bootout", &format!("gui/{}", uid), &daemonpath])
            .status()
        {
            Ok(_) => {
                println!("Daemon Stopped Successfully");
            }

            Err(_) => {
                println!("Error Stopping the daemon. Check if the daemon is running");
                return;
            }
        }
    } else if cfg!(target_os = "linux") {
        match Command::new("systemctl")
            .args(["--user", "stop", "docsearch.service"])
            .status()
        {
            Ok(_) => {
                println!("Daemon Stopped Successfully");
            }

            Err(_) => {
                println!("Error stopping the daemon. Check if it is running");
                return;
            }
        };
    } else {
        println!("No support for Windows. Get a better OS!!!");
    }
}
