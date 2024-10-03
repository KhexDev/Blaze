// #![windows_subsystem = "windows"]
use std::{collections::HashMap, process::Command, rc::Rc};

use disk_name::get_letters;
use ron::*;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};

slint::slint! {
    import { ScrollView, Button } from "std-widgets.slint";

    export enum EntryType {
        file,
        directory
    }

    export struct EntryItem {
        name: string,
        type_: EntryType,
        selected: bool,
    }

    export struct EntryChildren {
        entry: EntryItem,
        children: [EntryItem],
    }

    export global Global {
        in-out property <string> current-path;
        in-out property <[EntryChildren]> entries: [];
        in-out property <length> viewport-y;
    }

    export enum FastEntry {
        downloads,
        documents,
        pictures,
        music,
        videos,
        desktop,
    }

    export component App inherits Window {
        title: "Blaze by KhexDev";
        icon: @image-url("resources/icon.png");
        min-width: 900px;
        min-height: 720px;
        max-width: 900px;
        max-height: 720px;

        in property<[string]> files: [];
        in property<[EntryItem]> entries: [];

        property <[string]> fast_entries: [
            "Desktop",
            "Downloads",
            "Documents",
            "Pictures",
            "Music",
            "Videos",
        ];

        in-out property <[string]> disks_name: [];

        callback clicked-entry(EntryItem);
        callback selected-entry(EntryItem);
        callback go-back();
        callback reset_viewport_y();
        callback clicked-fast-entries(FastEntry);
        callback clicked-disk(string);

        forward-focus: key-handler;

        VerticalLayout {
            HorizontalLayout {
                alignment: center;
                height: 92%;

                Rectangle {
                    border-color: darkgray;
                    border-width: 1px;
                    width: 25%;
                    height: 100%;

                    VerticalLayout {
                        spacing: 8px;
                        alignment: start;

                        Text {
                            text: "Accès Rapide";
                        }

                        for name in fast_entries:
                        Rectangle {
                            HorizontalLayout {
                                padding-left: 10px;
                                spacing: 8px;
                                Image {
                                    source: @image-url("resources/icons/folder.png");
                                    width: 16px;
                                    height: 16px;
                                }
                                Text {
                                    text: name;
                                }
                            }
                            fast-entries-area := TouchArea {
                                clicked => {
                                    if name == "Desktop" {
                                        clicked-fast-entries(FastEntry.desktop);
                                    }
                                    if name == "Downloads" {
                                        clicked-fast-entries(FastEntry.downloads);
                                    }
                                    if name == "Documents" {
                                        clicked-fast-entries(FastEntry.documents);
                                    }
                                    if name == "Pictures" {
                                        clicked-fast-entries(FastEntry.pictures);
                                    }
                                    if name == "Music" {
                                        clicked-fast-entries(FastEntry.music);
                                    }
                                    if name == "Videos" {
                                        clicked-fast-entries(FastEntry.videos);
                                    }
                                 }
                            }
                            background: fast-entries-area.has-hover ? black : transparent;
                        }

                        Text {
                            text: "Disques";
                        }

                        for name in disks_name:
                        Rectangle {
                            HorizontalLayout {
                                padding-left: 10px;
                                spacing: 8px;

                                Image {
                                    source: @image-url("resources/icons/folder.png");
                                    width: 16px;
                                    height: 16px;
                                }
                                Text {
                                    text: name;
                                }
                            }
                            disk-area := TouchArea {
                                clicked => { clicked-disk(name); }
                            }
                            background: disk-area.has-hover ? black : transparent;
                        }
                    }
                }

                VerticalLayout {
                    HorizontalLayout {
                        Button {
                            text: " <- ";
                            // width: 24px;
                            height: 24px;
                            clicked => { go-back(); }
                        }
                        Text {
                            text: Global.current_path;
                            height: 24px;
                            width: 100%;
                        }
                    }

                    scroll-view := ScrollView {
                        width: 75%;
                        height: 96%;
                        viewport-y: Global.viewport-y;
                        enabled: true;

                        VerticalLayout {
                            padding-left: 8px;

                            for entry in entries:
                            Rectangle {
                                height: 24px;
                                width: 100%;

                                HorizontalLayout {
                                    spacing: 8px;
                                    Image {
                                        source: entry.type_ == EntryType.directory ? @image-url("resources/icons/folder.png") : @image-url("resources/icons/txt.png");
                                        width: 16px;
                                        height: 16px;
                                        cache-rendering-hint: true;
                                    }
                                    Text {
                                        text: entry.name;
                                        font-size: 18px;
                                        // padding: 100px;
                                        cache-rendering-hint: true;
                                    }
                                }
                                touch-area := TouchArea {
                                    double-clicked => {
                                        clicked_entry(entry);
                                        reset_viewport_y();
                                    }
                                    clicked => {
                                        debug("clicked");
                                        // entry.selected = true;
                                        // selected_entry(entry);
                                    }
                                }
                                background: entry.selected ? lightgray : touch-area.has-hover ? black : transparent;
                            }
                        }
                    }
                }
            }

            Rectangle {
                background: black;
                height: 8%;

                key-handler := FocusScope {
                    enabled: false;
                    key-pressed(event) => {
                        debug(event.text);
                        reject
                    }
                    focus-changed-event => {
                        debug("focus-changed-event");
                    }
                }

                HorizontalLayout {
                    Text {
                        text: "Status";
                        font-size: 24px;
                        vertical-alignment: bottom;
                        horizontal-alignment: left;
                    }
                    Text {
                        text: "Made by KhexDev";
                        font-size: 24px;
                        vertical-alignment: bottom;
                        horizontal-alignment: right;
                    }
                }
            }
        }
    }
}

enum LoadEntriesMethod {
    Next(String),
    Reload,
    Back,
}

#[derive(Serialize, Deserialize)]
struct EntriesCache {
    cache: HashMap<String, Vec<String>>,
    generated_by: String,
}

impl Default for EntriesCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            generated_by: "Blaze v0.1".into(),
        }
    }
}

fn load_entries(weak_app: &Weak<App>, method: LoadEntriesMethod) {
    let app = weak_app.upgrade().unwrap();
    let global = app.global::<Global>();

    match method {
        LoadEntriesMethod::Reload => {
            let path = app.global::<Global>().get_current_path().to_string();

            let files_readdir = std::fs::read_dir(path).unwrap();
            let mut entries_loaded = Vec::new();

            for file in files_readdir {
                if let Ok(entry) = file {
                    let type_ = if entry.file_type().unwrap().is_dir() {
                        EntryType::Directory
                    } else {
                        EntryType::File
                    };
                    let name: String = entry.file_name().to_str().unwrap().into();
                    let entry = EntryItem {
                        name: name.into(),
                        type_,
                        selected: false,
                    };
                    entries_loaded.push(entry);
                }
            }

            let entries = app.get_entries();
            let entries = entries
                .as_any()
                .downcast_ref::<VecModel<EntryItem>>()
                .expect("Failed to downcast");

            entries.set_vec(entries_loaded);
        }

        LoadEntriesMethod::Next(name) => {
            let mut new_path = global.get_current_path().to_string();

            if new_path.chars().filter(|c| *c == '\\').count() == 1 {
                if new_path.split('\\').collect::<Vec<&str>>().len() == 2 {
                    new_path.push_str(&format!("\\{}", name));
                } else {
                    new_path.push_str(&format!("{}", name));
                }
            } else {
                new_path.push_str(&format!("\\{}", name));
            }

            println!("new path {}", new_path);

            app.global::<Global>()
                .set_current_path(new_path.clone().into());

            let files_readdir = std::fs::read_dir(new_path).unwrap();
            let mut entries_loaded = Vec::new();

            for file in files_readdir {
                if let Ok(entry) = file {
                    let type_ = if entry.file_type().unwrap().is_dir() {
                        EntryType::Directory
                    } else {
                        EntryType::File
                    };
                    let name: String = entry.file_name().to_str().unwrap().into();
                    let entry = EntryItem {
                        name: name.into(),
                        type_,
                        selected: false,
                    };
                    entries_loaded.push(entry);
                }
            }

            let entries = app.get_entries();
            let entries = entries
                .as_any()
                .downcast_ref::<VecModel<EntryItem>>()
                .expect("Failed to downcast");

            entries.set_vec(entries_loaded);
        }
        LoadEntriesMethod::Back => {
            let mut new_path = global.get_current_path().to_string();
            if new_path.chars().filter(|c| *c == '\\').count() == 1 {
                new_path.replace_range(new_path.rfind('\\').unwrap() + 1.., "");
            } else {
                new_path.replace_range(new_path.rfind('\\').unwrap().., "");
            }

            println!("new path {}", new_path);

            app.global::<Global>()
                .set_current_path(new_path.clone().into());

            let files_readdir = std::fs::read_dir(new_path).unwrap();
            let mut entries_loaded = Vec::new();

            for file in files_readdir {
                if let Ok(entry) = file {
                    let type_ = if entry.file_type().unwrap().is_dir() {
                        EntryType::Directory
                    } else {
                        EntryType::File
                    };
                    let name: String = entry.file_name().to_str().unwrap().into();
                    let entry = EntryItem {
                        name: name.into(),
                        type_,
                        selected: false,
                    };
                    entries_loaded.push(entry);
                }
            }

            let entries = app.get_entries();
            let entries = entries
                .as_any()
                .downcast_ref::<VecModel<EntryItem>>()
                .expect("Failed to downcast");

            entries.set_vec(entries_loaded);
        }
    }
}

fn main() {
    let app = App::new().expect("Failed to create app");
    app.global::<Global>().set_current_path("C:\\".into());

    let disks_name_model = Rc::new(VecModel::from(
        get_letters()
            .iter()
            .map(|s| s.into())
            .collect::<Vec<SharedString>>(),
    ));
    let disks_name_model_rc = ModelRc::from(disks_name_model.clone());
    app.set_disks_name(disks_name_model_rc);

    let files_readdir = std::fs::read_dir("C:\\").unwrap();
    let mut entries_loaded = Vec::new();
    for file in files_readdir {
        if let Ok(entry) = file {
            let type_ = if entry.file_type().unwrap().is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            };
            let name: String = entry.file_name().to_str().unwrap().into();
            let entry = EntryItem {
                name: name.into(),
                type_,
                selected: false,
            };
            entries_loaded.push(entry);
        }
    }

    let entries = app.get_entries();
    let entries = entries
        .as_any()
        .downcast_ref::<VecModel<EntryItem>>()
        .expect("Failed to downcast");
    entries.set_vec(entries_loaded);

    let weak_app = app.as_weak();
    app.on_clicked_entry(move |entry| {
        // Load next entries
        if entry.type_ == EntryType::Directory {
            load_entries(&weak_app, LoadEntriesMethod::Next(entry.name.into()));
        } else {
            let app = weak_app.upgrade().unwrap();
            let mut path = app.global::<Global>().get_current_path().to_string();
            path.push_str(&format!("\\{}", &entry.name));
            open::that_detached(&path).expect("Failed to open file");
        }
    });

    let weak_app = app.as_weak();
    app.on_go_back(move || {
        load_entries(&weak_app, LoadEntriesMethod::Back);
    });

    let weak_app = app.as_weak();
    app.on_clicked_fast_entries(move |fast_entry| {
        let app = weak_app.upgrade().unwrap();
        match fast_entry {
            FastEntry::Downloads => app
                .global::<Global>()
                .set_current_path(dirs::download_dir().unwrap().to_str().unwrap().into()),
            FastEntry::Documents => app
                .global::<Global>()
                .set_current_path(dirs::document_dir().unwrap().to_str().unwrap().into()),
            FastEntry::Pictures => app
                .global::<Global>()
                .set_current_path(dirs::picture_dir().unwrap().to_str().unwrap().into()),
            FastEntry::Desktop => app
                .global::<Global>()
                .set_current_path(dirs::desktop_dir().unwrap().to_str().unwrap().into()),
            FastEntry::Music => app
                .global::<Global>()
                .set_current_path(dirs::audio_dir().unwrap().to_str().unwrap().into()),
            FastEntry::Videos => app
                .global::<Global>()
                .set_current_path(dirs::video_dir().unwrap().to_str().unwrap().into()),
        }
        println!(
            "NEW CURRENT PATH {}",
            app.global::<Global>().get_current_path()
        );
        load_entries(&weak_app, LoadEntriesMethod::Reload);
    });

    let weak_app = app.as_weak();
    app.on_clicked_disk(move |name| {
        let app = weak_app.upgrade().unwrap();
        app.global::<Global>().set_current_path(name);
        load_entries(&weak_app, LoadEntriesMethod::Reload);
    });

    app.run().expect("Failed to run app");
}
