//! Abstractions for the events passed back from i3.

use common;
use reply;
use serde_json as json;
use std::{fmt, str::FromStr};

use event::inner::*;

/// An event passed back from i3.
#[derive(Debug)]
pub enum Event {
    WorkspaceEvent(WorkspaceEventInfo),
    OutputEvent(OutputEventInfo),
    ModeEvent(ModeEventInfo),
    WindowEvent(WindowEventInfo),
    BarConfigEvent(BarConfigEventInfo),
    BindingEvent(BindingEventInfo),

    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    ShutdownEvent(ShutdownEventInfo),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkspaceEvent(event) => event.fmt(f),
            Self::WindowEvent(event) => event.fmt(f),
            Self::OutputEvent(event) => event.fmt(f),
            Self::ModeEvent(event) => event.fmt(f),
            Self::BarConfigEvent(event) => event.fmt(f),
            Self::BindingEvent(event) => event.fmt(f),

            #[cfg(feature = "i3-4-14")]
            #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
            Self::ShutdownEvent(event) => event.fmt(f),
        }
    }
}

/// Data for `WorkspaceEvent`.
#[derive(Debug)]
pub struct WorkspaceEventInfo {
    /// The type of change.
    pub change: WorkspaceChange,
    /// Will be `Some` if the type of event affects the workspace.
    pub current: Option<reply::Node>,
    /// Will be `Some` only when `change == Focus` *and* there was a previous workspace.
    /// Note that if the previous workspace was empty it will get destroyed when switching, but
    /// will still appear here.
    pub old: Option<reply::Node>,
}

impl fmt::Display for WorkspaceEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let current = if let Some(node) = &self.current {
            format!("{:?} ({})", node.name, node.id)
        } else {
            "unknown".to_string()
        };

        let old = if let Some(node) = &self.old {
            format!("{:?} ({})", node.name, node.id)
        } else {
            "unknown".to_string()
        };

        write!(f, "{:?} change from {} to {}", self.change, old, current)
    }
}

impl FromStr for WorkspaceEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        Ok(WorkspaceEventInfo {
            change: match val.get("change").unwrap().as_str().unwrap() {
                "focus" => WorkspaceChange::Focus,
                "init" => WorkspaceChange::Init,
                "empty" => WorkspaceChange::Empty,
                "urgent" => WorkspaceChange::Urgent,
                "rename" => WorkspaceChange::Rename,
                "reload" => WorkspaceChange::Reload,
                "move" => WorkspaceChange::Move,
                "restored" => WorkspaceChange::Restored,
                other => {
                    warn!(target: "i3ipc", "Unknown WorkspaceChange {}", other);
                    WorkspaceChange::Unknown
                }
            },
            current: match val.get("current").unwrap().clone() {
                json::Value::Null => None,
                val => Some(common::build_tree(&val)),
            },
            old: match val.get("old") {
                Some(o) => match o.clone() {
                    json::Value::Null => None,
                    val => Some(common::build_tree(&val)),
                },
                None => None,
            },
        })
    }
}

/// Data for `OutputEvent`.
#[derive(Debug)]
pub struct OutputEventInfo {
    /// The type of change.
    pub change: OutputChange,
}

impl fmt::Display for OutputEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} output event", self.change)
    }
}

impl FromStr for OutputEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        Ok(OutputEventInfo {
            change: match val.get("change").unwrap().as_str().unwrap() {
                "unspecified" => OutputChange::Unspecified,
                other => {
                    warn!(target: "i3ipc", "Unknown OutputChange {}", other);
                    OutputChange::Unknown
                }
            },
        })
    }
}

/// Data for `ModeEvent`.
#[derive(Debug)]
pub struct ModeEventInfo {
    /// The name of current mode in use. It is the same as specified in config when creating a
    /// mode. The default mode is simply named default.
    pub change: String,
}

impl fmt::Display for ModeEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} mode", self.change)
    }
}

impl FromStr for ModeEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        Ok(ModeEventInfo {
            change: val.get("change").unwrap().as_str().unwrap().to_owned(),
        })
    }
}

/// Data for `WindowEvent`.
#[derive(Debug)]
pub struct WindowEventInfo {
    /// Indicates the type of change
    pub change: WindowChange,
    /// The window's parent container. Be aware that for the "new" event, the container will hold
    /// the initial name of the newly reparented window (e.g. if you run urxvt with a shell that
    /// changes the title, you will still at this point get the window title as "urxvt").
    pub container: reply::Node,
}

impl fmt::Display for WindowEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} event for window {}; id: {}",
            self.change,
            self.container
                .name
                .as_ref()
                .unwrap_or(&"untitled".to_string()),
            self.container.id
        )
    }
}

impl FromStr for WindowEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        Ok(WindowEventInfo {
            change: match val.get("change").unwrap().as_str().unwrap() {
                "new" => WindowChange::New,
                "close" => WindowChange::Close,
                "focus" => WindowChange::Focus,
                "title" => WindowChange::Title,
                "fullscreen_mode" => WindowChange::FullscreenMode,
                "move" => WindowChange::Move,
                "floating" => WindowChange::Floating,
                "urgent" => WindowChange::Urgent,

                #[cfg(feature = "i3-4-13")]
                "mark" => WindowChange::Mark,

                other => {
                    warn!(target: "i3ipc", "Unknown WindowChange {}", other);
                    WindowChange::Unknown
                }
            },
            container: common::build_tree(val.get("container").unwrap()),
        })
    }
}

/// Data for `BarConfigEvent`.
#[derive(Debug)]
pub struct BarConfigEventInfo {
    /// The new i3 bar configuration.
    pub bar_config: reply::BarConfig,
}

impl fmt::Display for BarConfigEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bc = &self.bar_config;

        write!(
            f,
            "bar id: {}; mode: {}; position: {}; status command: {}",
            bc.id, bc.mode, bc.position, bc.status_command,
        )
    }
}

impl FromStr for BarConfigEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        Ok(BarConfigEventInfo {
            bar_config: common::build_bar_config(&val),
        })
    }
}

/// Data for `BindingEvent`.
///
/// Reports on the details of a binding that ran a command because of user input.
#[derive(Debug)]
pub struct BindingEventInfo {
    /// Indicates what sort of binding event was triggered (right now it will always be "run" but
    /// that may be expanded in the future).
    pub change: BindingChange,
    pub binding: Binding,
}

impl fmt::Display for BindingEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} '{}' {}+{}",
            self.change,
            self.binding.command,
            self.binding.event_state_mask.join("+"),
            self.binding.symbol.as_ref().unwrap_or(&"".to_string()),
        )
    }
}

impl FromStr for BindingEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        let bind = val.get("binding").unwrap();
        Ok(BindingEventInfo {
            change: match val.get("change").unwrap().as_str().unwrap() {
                "run" => BindingChange::Run,
                other => {
                    warn!(target: "i3ipc", "Unknown BindingChange {}", other);
                    BindingChange::Unknown
                }
            },
            binding: Binding {
                command: bind.get("command").unwrap().as_str().unwrap().to_owned(),
                event_state_mask: bind
                    .get("event_state_mask")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m.as_str().unwrap().to_owned())
                    .collect(),
                input_code: bind.get("input_code").unwrap().as_i64().unwrap() as i32,
                symbol: match bind.get("symbol").unwrap().clone() {
                    json::Value::String(s) => Some(s),
                    json::Value::Null => None,
                    _ => unreachable!(),
                },
                input_type: match bind.get("input_type").unwrap().as_str().unwrap() {
                    "keyboard" => InputType::Keyboard,
                    "mouse" => InputType::Mouse,
                    other => {
                        warn!(target: "i3ipc", "Unknown InputType {}", other);
                        InputType::Unknown
                    }
                },
            },
        })
    }
}

/// Data for `ShutdownEvent`.
#[derive(Debug)]
#[cfg(feature = "i3-4-14")]
#[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
pub struct ShutdownEventInfo {
    pub change: ShutdownChange,
}

#[cfg(feature = "i3-4-14")]
#[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
impl fmt::Display for ShutdownEventInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} event", self.change)
    }
}

#[cfg(feature = "i3-4-14")]
#[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
impl FromStr for ShutdownEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = json::from_str(s)?;
        let change = match val.get("change").unwrap().as_str().unwrap() {
            "restart" => ShutdownChange::Restart,
            "exit" => ShutdownChange::Exit,
            other => {
                warn!(target: "i3ipc", "Unknown ShutdownChange {}", other);
                ShutdownChange::Unknown
            }
        };
        Ok(ShutdownEventInfo { change })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reply::{Node, NodeBorder, NodeLayout, NodeType};
    use std::collections::HashMap;

    fn make_node() -> Node {
        reply::Node {
            focus: vec![],
            nodes: vec![],
            floating_nodes: vec![],
            id: 1234,
            name: Some("Firefox".to_string()),
            nodetype: NodeType::Root,
            border: NodeBorder::Normal,
            current_border_width: 2,
            layout: NodeLayout::Stacked,
            percent: None,
            rect: (0, 0, 1920, 1200),
            window_rect: (2, 0, 632, 366),
            deco_rect: (0, 0, 0, 0),
            geometry: (0, 0, 0, 0),
            window: None,
            window_properties: None,
            urgent: false,
            focused: true,

            #[cfg(feature = "i3-4-18-1")]
            marks: vec![],
        }
    }

    #[test]
    fn test_event_workspace_display() {
        let event = Event::WorkspaceEvent(WorkspaceEventInfo {
            change: WorkspaceChange::Empty,
            current: None,
            old: None,
        });
        assert_eq!(format!("{}", event), "Empty change from unknown to unknown");
    }

    #[test]
    fn test_event_output_display() {
        let event = Event::OutputEvent(OutputEventInfo {
            change: OutputChange::Unspecified,
        });
        assert_eq!(format!("{}", event), "unspecified output event");
    }

    #[test]
    fn test_event_mode_display() {
        let event = Event::ModeEvent(ModeEventInfo {
            change: "default".to_string(),
        });
        assert_eq!(format!("{}", event), "default mode");
    }

    #[test]
    fn test_event_window_display() {
        let event = Event::WindowEvent(WindowEventInfo {
            change: WindowChange::Focus,
            container: make_node(),
        });
        assert_eq!(
            format!("{}", event),
            "Focus event for window Firefox; id: 1234"
        );
    }

    #[test]
    fn test_event_bar_config_display() {
        let event = Event::BarConfigEvent(BarConfigEventInfo {
            bar_config: reply::BarConfig {
                id: "mybar".to_string(),
                mode: "dock".to_string(),
                position: "top".to_string(),
                status_command: "i3blocks".to_string(),
                font: "Helvetica".to_string(),
                workspace_buttons: true,
                binding_mode_indicator: true,
                verbose: false,
                colors: HashMap::new(),
            },
        });
        assert_eq!(
            format!("{}", event),
            "bar id: mybar; mode: dock; position: top; status command: i3blocks"
        );
    }

    #[test]
    fn test_event_binding_display() {
        let event = Event::BindingEvent(BindingEventInfo {
            change: BindingChange::Run,
            binding: Binding {
                command: r#"[con_mark="F1"] focus"#.to_string(),
                event_state_mask: vec!["Mod4".to_string()],
                input_code: 0,
                symbol: Some("F1".to_string()),
                input_type: InputType::Keyboard,
            },
        });
        assert_eq!(
            format!("{}", event),
            r#"Run '[con_mark="F1"] focus' Mod4+F1"#
        );
    }

    #[test]
    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    fn test_event_shutdown_display() {
        let event = Event::ShutdownEvent(ShutdownEventInfo {
            change: ShutdownChange::Restart,
        });
        assert_eq!(format!("{}", event), "restart event");
    }
}

/// Less important types
pub mod inner {
    use std::fmt;

    /// The kind of workspace change.
    #[derive(Debug, PartialEq)]
    pub enum WorkspaceChange {
        Focus,
        Init,
        Empty,
        Urgent,
        Rename,
        Reload,
        Restored,
        Move,
        /// A WorkspaceChange we don't support yet.
        Unknown,
    }

    /// The kind of output change.
    #[derive(Debug, PartialEq)]
    pub enum OutputChange {
        Unspecified,
        /// An OutputChange we don't support yet.
        Unknown,
    }

    impl fmt::Display for OutputChange {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Unspecified => "unspecified",
                    Self::Unknown => "unknown",
                }
            )
        }
    }

    /// The kind of window change.
    #[derive(Debug, PartialEq)]
    pub enum WindowChange {
        /// The window has become managed by i3.
        New,
        /// The window has closed>.
        Close,
        /// The window has received input focus.
        Focus,
        /// The window's title has changed.
        Title,
        /// The window has entered or exited fullscreen mode.
        FullscreenMode,
        /// The window has changed its position in the tree.
        Move,
        /// The window has transitioned to or from floating.
        Floating,
        /// The window has become urgent or lost its urgent status.
        Urgent,

        /// A mark has been added to or removed from the window.
        #[cfg(feature = "i3-4-13")]
        #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-13")))]
        Mark,

        /// A WindowChange we don't support yet.
        Unknown,
    }

    /// Either keyboard or mouse.
    #[derive(Debug, PartialEq)]
    pub enum InputType {
        Keyboard,
        Mouse,
        /// An InputType we don't support yet.
        Unknown,
    }

    /// Contains details about the binding that was run.
    #[derive(Debug, PartialEq)]
    pub struct Binding {
        /// The i3 command that is configured to run for this binding.
        pub command: String,

        /// The group and modifier keys that were configured with this binding.
        pub event_state_mask: Vec<String>,

        /// If the binding was configured with blindcode, this will be the key code that was given for
        /// the binding. If the binding is a mouse binding, it will be the number of times the mouse
        /// button was pressed. Otherwise it will be 0.
        pub input_code: i32,

        /// If this is a keyboard binding that was configured with bindsym, this field will contain the
        /// given symbol. Otherwise it will be None.
        pub symbol: Option<String>,

        /// Will be Keyboard or Mouse depending on whether this was a keyboard or mouse binding.
        pub input_type: InputType,
    }

    /// The kind of binding change.
    #[derive(Debug, PartialEq)]
    pub enum BindingChange {
        Run,
        /// A BindingChange we don't support yet.
        Unknown,
    }

    impl fmt::Display for BindingChange {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Run => "run",
                    Self::Unknown => "unknown",
                }
            )
        }
    }

    /// The kind of shutdown change.
    #[derive(Debug, PartialEq)]
    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    pub enum ShutdownChange {
        Restart,
        Exit,
        /// A ShutdownChange we don't support yet.
        Unknown,
    }

    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    impl fmt::Display for ShutdownChange {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Restart => "restart",
                    Self::Exit => "exit",
                    Self::Unknown => "unknown",
                }
            )
        }
    }
}
