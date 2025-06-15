use crate::{app::App, tabs::Tab};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    process::Command,
};

pub fn com_hi(app: &mut App, args: Vec<String>) {
    app.throw_status_message("Hello!".to_string());
    return;
}

pub fn com_w(app: &mut App, args: Vec<String>) {
    let curtab = &mut app.tabs[app.cur_tab];
    let mut file_out_name: String = String::new();
    if !args.is_empty() {
        file_out_name = args[0].clone();
    } else if !curtab.filename.is_empty() {
        file_out_name = curtab.filename.clone();
    } else {
        app.throw_status_message("Usage: !w filename".to_string());
        return;
    }

    let mut file_out: File = match File::create(file_out_name.clone()) {
        Ok(f) => f,
        Err(e) => {
            app.throw_status_message(e.to_string());
            return;
        }
    };

    curtab.filename = file_out_name;
    curtab.changed = false;
    let mut contents: String = curtab.buf.join("\n");
    contents.push('\n');
    match file_out.write_all(contents.as_bytes()) {
        Ok(_) => app.throw_status_message("Success".to_string()),
        Err(e) => {
            curtab.changed = true;
            app.throw_status_message(e.to_string());
        }
    };
}

pub fn com_r(app: &mut App, args: Vec<String>) {
    let curtab = &mut app.tabs[app.cur_tab];
    if curtab.changed {
        app.throw_status_message("W: Current buffer isn't saved. !ri to ignore".to_string());
        return;
    }
    if args.is_empty() {
        app.throw_status_message("Usage: !r filename".to_string());
        return;
    }

    let file_in: File = match File::open(args[0].clone()) {
        Ok(f) => f,
        Err(e) => {
            app.throw_status_message(e.to_string());
            return;
        }
    };

    curtab.buf.clear();
    let reader = BufReader::new(file_in);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let res = l.clone().replace('\n', "");
                curtab.buf.push(res);
            }
            Err(e) => {
                curtab.buf.clear();
                app.throw_status_message(e.to_string());
                return;
            }
        }
    }
    curtab.changed = false;
    curtab.cursor_xy = (0, 0);
    curtab.filename = args[0].clone();
    app.throw_status_message("Success".to_string());
    return;
}

pub fn com_ri(app: &mut App, args: Vec<String>) {
    let curtab = &mut app.tabs[app.cur_tab];
    if args.is_empty() {
        app.throw_status_message("Usage: !r filename".to_string());
        return;
    }

    let file_in: File = match File::open(args[0].clone()) {
        Ok(f) => f,
        Err(e) => {
            app.throw_status_message(e.to_string());
            return;
        }
    };

    curtab.buf.clear();
    let reader = BufReader::new(file_in);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let res = l.clone().replace('\n', "");
                curtab.buf.push(res);
            }
            Err(e) => {
                curtab.buf.clear();
                app.throw_status_message(e.to_string());
                return;
            }
        }
    }
    curtab.changed = false;
    curtab.cursor_xy = (0, 0);
    curtab.filename = args[0].clone();
    app.throw_status_message("Success".to_string());
    return;
}

pub fn com_rn(app: &mut App, args: Vec<String>) {
    if args.is_empty() {
        app.throw_status_message("Usage: !rn filename".to_string());
        return;
    }
    let filename = args[0].clone();
    let mut newtab = Tab::new(Some(filename.clone()));

    match newtab.readf(filename.clone()) {
        Ok(()) => {}
        Err(e) => {
            app.throw_status_message(e.to_string());
            return;
        }
    }
    newtab.filename = filename;
    app.tabs.push(newtab);
    app.cur_tab = app.tabs.len().saturating_sub(1);
    app.throw_status_message("Success".to_string());
    return;
}

pub fn com_q(app: &mut App, args: Vec<String>) {
    let curtab = &app.tabs[app.cur_tab];
    if curtab.changed {
        app.throw_status_message(
            "W: Current buffer has unsaved changes; !qi to ignore".to_string(),
        );
        return;
    }
    app.running = false;
}

pub fn com_qi(app: &mut App, args: Vec<String>) {
    app.running = false;
}

pub fn com_exec(app: &mut App, args: Vec<String>) {
    if args.is_empty() {
        app.throw_status_message("Usage: !exec command".to_string());
        return;
    }
    let com = if cfg!(target_os = "windows") {
        let argline: &str = &args.join(" ");
        Command::new("cmd")
            .args(&["/C", &argline])
            .output()
            .expect("Error creating cmd")
    } else {
        let argline: &str = &args.join(" ");
        Command::new("sh")
            .args(&["-c", argline])
            .output()
            .expect("Error running sh")
    };

    let mut output_s: String = String::new();
    if com.stdout.is_empty() {
        output_s = String::from_utf8_lossy(&com.stderr).to_string();
    } else {
        output_s = String::from_utf8_lossy(&com.stdout).to_string();
    }
    app.throw_status_message(output_s);
}

pub fn com_exec_f(app: &mut App, args: Vec<String>) {
    // executes shell/cmd script from file (current or specified)
    if args.is_empty() {
        app.throw_status_message("Usage: !exec_f filename".to_string());
        return;
    }

    let com = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(args)
            .output()
            .expect("Error creating cmd")
    } else {
        Command::new("sh")
            .args(args)
            .output()
            .expect("Error running sh")
    };

    let mut output_s: String = String::new();
    if com.stdout.is_empty() {
        output_s = String::from_utf8_lossy(&com.stderr).to_string();
    } else {
        output_s = String::from_utf8_lossy(&com.stdout).to_string();
    }
    app.throw_status_message(output_s);
}

pub fn com_execn(app: &mut App, args: Vec<String>) {
    let mut same_tab: bool = false;
    let mut ignore_flag: bool = false;

    if args.is_empty() {
        app.throw_status_message("Usage: !execn command".to_string());
        return;
    }
    if args.get(0) == Some(&"~cur".to_string()) {
        same_tab = true;
        if args.get(1) == Some(&"~ignore".to_string()) {
            ignore_flag = true;
        }
    }

    let argline: &str = match same_tab {
        true => match ignore_flag {
            true => &args[2..].join(" "),
            false => &args[1..].join(" "),
        },
        false => &args.join(" "),
    };
    let com = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &argline])
            .output()
            .expect("Error creating cmd")
    } else {
        Command::new("sh")
            .args(&["-c", argline])
            .output()
            .expect("Error running sh")
    };

    let mut output_s: String = String::new();
    if com.stdout.is_empty() {
        output_s = String::from_utf8_lossy(&com.stderr).to_string();
    } else {
        output_s = String::from_utf8_lossy(&com.stdout).to_string();
    }

    let lines: Vec<String> = output_s.lines().map(|line| line.to_string()).collect();

    if same_tab {
        if let Some(tab) = app.tabs.get_mut(app.cur_tab) {
            if (tab.changed && !ignore_flag) {
                app.throw_status_message(
                    "W: This tab has unsaved changes. ~ignore to ignore".to_owned(),
                );
                return;
            }
            tab.buf = lines;
        }
    } else {
        let mut output_tab = Tab::new(Some("Output".to_string()));
        output_tab.buf = lines;
        app.tabs.push(output_tab);
        app.cur_tab = app.tabs.len().saturating_sub(1);
    }

    app.throw_status_message("Success".to_string());
    return;
}

pub fn com_execn_f(app: &mut App, args: Vec<String>) {
    let mut same_tab: bool = false;
    let mut ignore_flag: bool = false;

    if args.is_empty() {
        app.throw_status_message("Usage: !execn command".to_string());
        return;
    }
    if args.get(0) == Some(&"~cur".to_string()) {
        same_tab = true;
        if args.get(1) == Some(&"~ignore".to_string()) {
            ignore_flag = true;
        }
    }
    let argline: &str = match same_tab {
        true => match ignore_flag {
            true => &args[2..].join(" "),
            false => &args[1..].join(" "),
        },
        false => &args.join(" "),
    };
    let com = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args([argline])
            .output()
            .expect("Error creating cmd")
    } else {
        Command::new("sh")
            .args([argline])
            .output()
            .expect("Error running sh")
    };

    let mut output_s: String = String::new();
    if com.stdout.is_empty() {
        output_s = String::from_utf8_lossy(&com.stderr).to_string();
    } else {
        output_s = String::from_utf8_lossy(&com.stdout).to_string();
    }
    let lines: Vec<String> = output_s.lines().map(|line| line.to_string()).collect();

    if same_tab {
        if let Some(tab) = app.tabs.get_mut(app.cur_tab) {
            if (tab.changed && !ignore_flag) {
                app.throw_status_message(
                    "W: This tab has unsaved changes. ~ignore to ignore".to_owned(),
                );
                return;
            }
            tab.buf = lines;
        }
    } else {
        let mut output_tab = Tab::new(Some("Output".to_string()));
        output_tab.buf = lines;
        app.tabs.push(output_tab);
        app.cur_tab = app.tabs.len().saturating_sub(1);
    }

    app.throw_status_message("Success".to_string());
    return;
}

pub fn com_tab(app: &mut App, args: Vec<String>) {
    if args.is_empty() {
        app.throw_status_message(
            "Usage: !tab new, !tab goto num, !tab rm num, !tab next, !tab prev, !tab rename num name".to_string(),
        );
        return;
    }
    if args[0] == "new" {
        app.tabs.push(Tab::new(None));
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "goto" {
        let ind: usize = match args[1].parse() {
            Ok(n) => n,
            Err(e) => {
                app.throw_status_message(e.to_string());
                return;
            }
        };
        if ind > app.tabs.len() {
            app.throw_status_message("Tab with specified indice not opened".to_string());
            return;
        }
        app.cur_tab = ind.saturating_sub(1);
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "rm" {
        let mut ind: usize = match args[1].parse() {
            Ok(n) => n,
            Err(e) => {
                app.throw_status_message(e.to_string());
                return;
            }
        };
        ind = ind.saturating_sub(1);
        if ind >= app.tabs.len() {
            app.throw_status_message("Tab with specified indice not opened".to_string());
            return;
        }
        app.tabs.remove(ind);
        if app.tabs.len() == 0 {
            let newtab = Tab::new(None);
            app.tabs.push(newtab);
            app.cur_tab = 0;
        } else if app.cur_tab >= app.tabs.len() {
            app.cur_tab = app.cur_tab.saturating_sub(1);
        }
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "next" {
        if app.cur_tab + 1 >= app.tabs.len() {
            app.throw_status_message("Current tab is already last!".to_string());
            return;
        }
        app.cur_tab += 1;
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "prev" {
        if app.cur_tab == 0 {
            app.throw_status_message("Current tab is first!".to_string());
            return;
        }
        app.cur_tab -= 1;
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "rename" {
        let ind: usize = match args[1].parse() {
            Ok(n) => n,
            Err(e) => {
                app.throw_status_message(e.to_string());
                return;
            }
        };
        if ind > app.tabs.len() {
            app.throw_status_message("Tab with specified indice not opened".to_string());
            return;
        }
        let new_name: String = args[2..].join(" ");
        app.tabs[ind.saturating_sub(1)].displayed_name = new_name;
        app.throw_status_message("Success".to_string());
        return;
    }
    if args.get(0) == Some(&"left".to_string()) {
        app.left_area_open = !app.left_area_open;
        app.throw_status_message("success".to_owned());
        return;
    }
    if args.get(0) == Some(&"leftuse".to_string()) {
        app.left_area_used = !app.left_area_used;
        app.throw_status_message("success".to_owned());
        return;
    }
    app.throw_status_message(
        "Usage: !tab new, !tab goto num, !tab rm num, !tab next, !tab prev, !tab rename num name"
            .to_string(),
    );
    return;
}

pub fn com_version(app: &mut App, args: Vec<String>) {
    app.throw_status_message(app.version.clone());
    return;
}

pub fn com_alias(app: &mut App, args: Vec<String>) {
    if args.is_empty() {
        app.throw_status_message("Usage: !alias new / !alias rm".to_string());
        return;
    }
    if args[0] == "new" {
        if args.len() < 3 {
            app.throw_status_message("Usage: !alias new alias_name command".to_string());
            return;
        }
        app.aliases.insert(args[1].clone(), args[2..].to_vec());
        app.throw_status_message("Success".to_string());
        return;
    }
    if args[0] == "rm" {
        if args.len() < 2 {
            app.throw_status_message("Usage: !alias rm alias_name".to_string());
            return;
        }
        match app.aliases.remove(&args[1]) {
            Some(_) => {
                app.throw_status_message("Success".to_string());
                return;
            }
            None => {
                app.throw_status_message("No alias with this name was saved".to_string());
                return;
            }
        }
    }
}
