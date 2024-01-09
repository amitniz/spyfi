use super::screens::colorscheme::Theme;
use std::io::Stdout;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Rect, Constraint, Direction, Layout,Alignment},
    widgets::{self,Block,Borders,ListItem,ListState,Wrap, Tabs},
    text::{Span,Spans},
    style::{Color,Style,Modifier},
    Frame
};


pub trait Item<B:Backend>{
    fn render(&mut self,frame: &mut Frame<B>, area: Rect) where B:Backend;
}


// ----------------------------------- Pane -----------------------------------

type DynamicItem<B> = Box<dyn Item<B>>;

pub struct Pane<B:Backend>{
    title: Option<String>,
    items:Vec<DynamicItem<B>>,
    constraints: Vec<Constraint>,
    direction: Direction,
}


impl<B:Backend> Item<B> for Pane<B> where B:Backend{
    fn render(&mut self,frame: &mut Frame<B>, area: Rect){

        let chunks = Layout::default()
            .direction(self.direction.clone())
            .constraints(self.constraints.clone())
            .split(area);
    
        for i in 0..self.items.len(){
            self.items[i].render(frame, chunks[i]); 
        }
    }
}

impl<B:Backend> Pane<B>{

    pub fn new(title:Option<&str>,theme:&Theme) -> Self{
        Pane{
            title: title.map(str::to_string),
            items: vec![],
            constraints:vec![],
            direction: Direction::Vertical,
        }
    }
    
    pub fn split(&mut self,axis:Direction,sections:&Vec<u16>) -> &mut Self{
        self.constraints = sections.iter()
            .map(|&i|{Constraint::Percentage(i)})
            .collect();
        self
    }

    pub fn items(&mut self,items: Vec<DynamicItem<B>>) -> &mut Self{
        self.items = items;
        self
    }
}

// ------------------------------- Paragraph ----------------------------------

#[derive(Clone)]
pub struct Paragraph<'a>{
    lines: Vec<Spans<'a>>,
    borders: Borders,
    alignment: Alignment,
    style:Style,
}

impl<'a,B:Backend> Item<B> for Paragraph<'a>{
    
    fn render(&mut self,frame: &mut Frame<B>, area: Rect) where B:Backend{
        let block = widgets::Paragraph::new(self.lines.clone())
            .block( Block::default()
                    .borders(self.borders)
                    .style(self.style)
                    
            )
            .alignment(self.alignment);
        frame.render_widget(block,area);
    }
}

impl<'a> Paragraph<'a>{
    pub fn new(lines: Vec<&'static str>) -> Self{
        
        let lines = lines.iter().map(|line| Spans::from(line.to_owned())).collect::<Vec<Spans>>();

        Paragraph{
            lines,
            borders:Borders::NONE,
            alignment:Alignment::Left,
            style:Style::default(),
        }
    }

    pub fn borders(&mut self,border:Borders) -> &mut Self{
        self.borders = border;
        self
    }
   
    pub fn style(&mut self, style:Style) -> &mut Self{
        self.style = style;
        self
    }
    pub fn alignment(&mut self,alignment: Alignment) -> &mut Self{
        self.alignment = alignment;
        self
    }
}

// ------------------------------- List ----------------------------------
#[derive(Clone)]
pub struct List<'a>{
    state: ListState,
    items: Vec<ListItem<'a>>,
    borders: Borders,
    style: Style,
}

impl<'a> List<'a>{
    pub fn new(items: Vec<&'static str>) -> Self{
        List{
            state: ListState::default(),
            items: items.iter().map(|i|ListItem::new(format!(" {} ",i))).collect(),
            borders: Borders::NONE,
            style:Style::default(),
        }
    }

    pub fn borders(&mut self,border:Borders) -> &Self{
        self.borders = border;
        self
    }
    
    pub fn style(&mut self, style:Style) -> &mut Self{
        self.style = style;
        self
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

}

impl<'a,B:Backend> Item<B> for List<'a>{
    fn render(&mut self,frame: &mut Frame<B>, area: Rect) where B:Backend{
        
        let block = widgets::List::new(self.items.clone())
            .block( Block::default()
                    .borders(self.borders)
                    .style(self.style) 
            );
        frame.render_stateful_widget(block,area,&mut self.state);
    }
}

