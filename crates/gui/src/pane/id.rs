#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaneID
{
    #[default]
    Parent,
    Child,
}
impl std::fmt::Display for PaneID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Self::Parent => {
                if f.alternate() {
                    write!(f, "Parent")
                } else {
                    write!(f, "parent")
                }
            }
            Self::Child => {
                if f.alternate() {
                    write!(f, "Child")
                } else {
                    write!(f, "child")
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaneSelection
{
    #[default]
    ActivePane,
    BothPanes,
    Id(PaneID),
}
impl std::fmt::Display for PaneSelection
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            if let Self::Id(pane_id) = self {
                write!(f, " {pane_id:#}")
            } else {
                write!(f, "")
            }
        } else {
            match self {
                Self::ActivePane => write!(f, " active pane"),
                Self::BothPanes => write!(f, " both panes"),
                Self::Id(pane_id) => write!(f, " {pane_id}"),
            }
        }
    }
}
