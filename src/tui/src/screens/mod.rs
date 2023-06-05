use crossterm::event::{KeyEvent,KeyCode};
use super::monitor;

use tui::{
    backend::Backend,
    layout::{Rect, Constraint, Direction, Layout},
    widgets::{Block, Borders,Paragraph,ListItem,ListState,List,Wrap, Tabs},
    text::{Span,Spans},
    style::{Color,Style,Modifier},
    Frame
};

pub mod colorscheme;
use colorscheme::Theme;

pub trait Screen<B:Backend>{
    /// Sets a layout for a given frame    
    fn set_layout(&mut self, f: &mut Frame<B>);
    /// handle keyboard event. If uncatched return false
    fn handle_input(&mut self, key:KeyEvent) -> bool;
    fn set_theme(&mut self, theme:Theme);
    fn theme_name(&mut self) -> &str;
    fn update(&mut self,ipc_msg:monitor::IPCMessage);
}

// ------------------------------ import screens ------------------------------
pub mod welcome_screen;
pub use welcome_screen::*;
pub mod main_screen;
pub use main_screen::*;


// ------------------------------  custom widgets -----------------------------

#[derive(Default)]
struct StatefulList<T>{
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T>{
    fn new(items:Vec<T>) -> StatefulList<T>{
        let mut state_list = StatefulList{
            state: ListState::default(),
            items,
        };
        state_list.state.select(Some(0));
        state_list
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

}
