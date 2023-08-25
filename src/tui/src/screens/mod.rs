use std::collections::HashMap;

use crossterm::event::{KeyEvent,KeyCode};
use wpa::NetworkInfo;
use threads::ipc::IPCMessage;
use tui::{
    backend::Backend,
    layout::{Rect, Constraint, Direction, Layout,Alignment},
    widgets::{Block, Borders,Paragraph,ListItem,ListState,List,Wrap, Clear,Gauge},
    text::{Span,Spans},
    style::{Color,Style,Modifier},
    Frame
};

pub mod colorscheme;
use colorscheme::Theme;

pub type ScreenIPC = IPCMessage<HashMap<String,NetworkInfo>>;

pub trait Screen<B:Backend>{
    /// Sets a layout for a given frame    
    fn set_layout(&mut self, f: &mut Frame<B>);
    /// handle keyboard event. If uncatched return false
    fn handle_input(&mut self, key:KeyEvent) -> bool;
    fn update(&mut self,ipc_msg:  ScreenIPC) -> Option<ScreenIPC>;
    fn set_theme(&mut self, theme: &Theme);
}

// ------------------------------ import screens ------------------------------
pub mod welcome_screen;
pub use welcome_screen::*;
pub mod main_screen;
pub use main_screen::*;


// ------------------------------  custom widgets -----------------------------

#[derive(Default,Debug,Clone)]
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

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn selected(&self) -> Option<&T>{
        match self.state.selected(){
            Some(i) => Some(&self.items[i]),
            None => None,
        }
    }

}

#[derive(Default)]
pub struct Panes{
    selected: usize,
    panes: Vec<String>,
}

impl Panes{
    //add pane, return false if exists already
    pub fn add_pane(&mut self,name: &str) -> bool{
        if !self.panes.contains(&name.to_owned()){
            self.panes.push(name.to_owned());
            return true;
        }
        return false;
    }
    
    //remove a pane, return false if not exists
    pub fn remove_pane(&mut self,name: &str) -> bool{
        let init_size = self.panes.len();
        self.panes = self.panes.iter()
            .filter(|&i| i != &name.to_owned())
            .map(Clone::clone)
            .collect();
        init_size != self.panes.len() 
    }

    pub fn selected(&self) -> String{
        if self.panes.len() > self.selected{
            return self.panes[self.selected].clone();
        }
        "".to_owned()
    }

    pub fn next(&mut self){
        self.selected = aux::modulos((self.selected+1) as i32, self.panes.len() as i32) as usize;
    }
}

