extern crate sdl2;
extern crate rand;

use sdl2::VideoSubsystem;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use std::thread::sleep;
use std::time::{Instant,Duration};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, LineWriter};
use std::path::Path;
use std::collections::HashMap;

use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};

// Define some constants
const WIN_WIDTH: i32 = 480;
const WIN_HEIGHT: i32 = 560;
const NB_ROWS: i32 = 20;
const NB_COLUMNS: i32 = 12;
const LEFT: i32 = 10;
const TOP: i32 = 10;
const CELL_SIZE: i32 = (WIN_WIDTH / (NB_COLUMNS + 7)) as i32;

static TETRIS_COLORS: [Color; 8] = [
    Color{r: 0x0,g: 0x0,b: 0x0,a: 0x0},
    Color{r: 0xFF,g: 0x60,b: 0x60,a: 0xFF},
    Color{r: 0x60,g: 0xFF,b: 0x60,a: 0xFF},
    Color{r: 0x60,g: 0x60,b: 0xFF,a: 0xFF},
    Color{r: 0xCC,g: 0xCC,b: 0x60,a: 0xFF},
    Color{r: 0xCC,g: 0x60,b: 0xCC,a: 0xFF},
    Color{r: 0x60,g: 0xCC,b: 0xCC,a: 0xFF},
    Color{r: 0xDA,g: 0xAA,b: 0x00,a: 0xFF},
];

#[derive(Clone, Copy)]
struct Vector2i{
    x : i32,
    y : i32,
}

#[derive(Clone, Copy)]
struct TetrisShape {
    x: i32,
    y: i32,
    typ: i32,
    v: [Vector2i; 4],
}

impl TetrisShape {
    fn new(x: i32, y: i32, typ: i32) -> Self {
        let mut s = TetrisShape {
            x,
            y,
            typ,
            v: [Vector2i { x: 0, y: 0 }; 4],
        };
        s.init_shape(typ);
        s
    }

    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            typ: 0,
            v: [Vector2i { x: 0, y: 0 }; 4],
        }
    }

    fn init(&mut self, x: i32, y: i32, typ: i32) {
        self.x = x;
        self.y = y;
        self.typ = typ;
        self.init_shape(typ);
    }

    fn init_shape(&mut self, typ: i32) {
        match typ {
            1 => {
                self.v = [
                    Vector2i { x: 0, y: -1 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: -1, y: 0 },
                    Vector2i { x: -1, y: 1 },
                ];
            }
            2 => {
                self.v = [
                    Vector2i { x: 0, y: -1 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 1, y: 0 },
                    Vector2i { x: 1, y: 1 },
                ];
            }
            3 => {
                self.v = [
                    Vector2i { x: 0, y: -1 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 0, y: 1 },
                    Vector2i { x: 0, y: 2 },
                ];
            }
            4 => {
                self.v = [
                    Vector2i { x: -1, y: 0 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 1, y: 0 },
                    Vector2i { x: 0, y: 1 },
                ];
            }
            5 => {
                self.v = [
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 1, y: 0 },
                    Vector2i { x: 0, y: 1 },
                    Vector2i { x: 1, y: 1 },
                ];
            }
            6 => {
                self.v = [
                    Vector2i { x: -1, y: -1 },
                    Vector2i { x: 0, y: -1 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 0, y: 1 },
                ];
            }
            7 => {
                self.v = [
                    Vector2i { x: 1, y: -1 },
                    Vector2i { x: 0, y: -1 },
                    Vector2i { x: 0, y: 0 },
                    Vector2i { x: 0, y: 1 },
                ];
            }
            _ => {}
        }
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) {

        if self.typ != 0{
            //--
            let mut l: i32;
            let mut t: i32;
            let c = TETRIS_COLORS[self.typ as usize];
            canvas.set_draw_color(c);

            let a = (CELL_SIZE-2) as u32;
            for p in self.v {
                t = (self.y + p.y*CELL_SIZE + TOP + 1) as i32;
                l = (self.x + p.x*CELL_SIZE + LEFT + 1) as i32;
                if t>=0 {
                    canvas.fill_rect(Rect::new(l,t, a,a));
                }
            }
        }

    }

    fn rotate_left(&mut self) {
        //--
        if self.typ != 5 {
            let mut x: i32;
            let mut y: i32;
            for i in 0..4 {
                x = self.v[i].y;
                y = -self.v[i].x;
                self.v[i].x = x;
                self.v[i].y = y;
            }
        }
    }

    fn rotate_right(&mut self) {
        //--
        if self.typ != 5 {
            let mut x;
            let mut y;
            for i in 0..4 {
                x = -self.v[i].y;
                y = self.v[i].x;
                self.v[i].x = x;
                self.v[i].y = y;
            }
        }
    }

    fn is_in_board(&self) -> bool {
        let mut x: i32;
        let mut y: i32;
        for i in 0..4 {
            x = self.v[i].x + self.x;
            y = self.v[i].y + self.y;
            if (x < 0) || (x >= NB_COLUMNS) || (y >= NB_ROWS) {
                return false;
            }
        }
        true
    }

    fn is_out_left(&self) -> bool {
        let l : i32 = self.min_x_v()*CELL_SIZE + self.x;
        return (l<0);
    }

    fn is_out_right(&self) -> bool {
        let r : i32 = self.max_x_v()*CELL_SIZE + CELL_SIZE + self.x;
        return (r>(NB_COLUMNS*CELL_SIZE));
    }

    fn is_out_bottom(&self) -> bool {
        let b : i32 = self.max_y_v()*CELL_SIZE + CELL_SIZE + self.y;
        return (b>NB_ROWS*CELL_SIZE);
    }

    fn hit_ground1(&self, board: &[i32; (NB_ROWS * NB_COLUMNS) as usize]) -> bool {
        let mut ix : i32;
        let mut iy : i32;
        let mut x : i32;
        let mut y : i32;

        for p in self.v {

            x  = p.x * CELL_SIZE + self.x + 1;
            y  = p.y * CELL_SIZE + self.y + 1;
            ix = x / CELL_SIZE;
            iy = y / CELL_SIZE;
            if (ix>=0) && (ix<NB_COLUMNS) && (iy>=0) && (iy<NB_ROWS) {
                //println!("ix = {:?} iy = {:?}",ix,iy);
                if board[(iy*NB_COLUMNS + ix) as usize]!=0{
                    return true;
                }
            }

            x  = p.x * CELL_SIZE + CELL_SIZE - 1 + self.x;
            y  = p.y * CELL_SIZE + self.y + 1;
            ix = x / CELL_SIZE;
            iy = y / CELL_SIZE;
            if (ix>=0) && (ix<NB_COLUMNS) && (iy>=0) && (iy<NB_ROWS) {
                if board[(iy*NB_COLUMNS + ix) as usize]!=0{
                    return true;
                }
            }

            x  = p.x * CELL_SIZE + CELL_SIZE - 1 + self.x;
            y  = p.y * CELL_SIZE + CELL_SIZE - 1 + self.y;
            ix = x / CELL_SIZE;
            iy = y / CELL_SIZE;
            if (ix>=0) && (ix<NB_COLUMNS) && (iy>=0) && (iy<NB_ROWS) {
                if board[(iy*NB_COLUMNS + ix) as usize]!=0{
                    return true;
                }
            }

            x  = p.x * CELL_SIZE + self.x + 1;
            y  = p.y * CELL_SIZE + CELL_SIZE - 1 + self.y;
            ix = x / CELL_SIZE;
            iy = y / CELL_SIZE;
            if (ix>=0) && (ix<NB_COLUMNS) && (iy>=0) && (iy<NB_ROWS) {
                if board[(iy*NB_COLUMNS + ix) as usize]!=0{
                    return true;
                }
            }

        }

        return false;
    }

    fn max_x(&self)->i32{
        let mut maxi = self.v[0].x + self.x;
        for i in 1..4{
            let x = self.v[i].x + self.x;
            if x>maxi{
                maxi = x;
            }
        }
        maxi
    }

    fn max_x_v(&self)->i32{
        let mut maxi = self.v[0].x;
        for i in 1..4{
            let x = self.v[i].x;
            if x>maxi{
                maxi = x;
            }
        }
        maxi
    }

    fn min_x(&self)->i32{
        let mut mini = self.v[0].x+ self.x;
        for i in 1..4{
            let x = self.v[i].x+ self.x;
            if x<mini{
                mini = x;
            }
        }
        mini
    }

    fn min_x_v(&self)->i32{
        let mut mini = self.v[0].x;
        for i in 1..4{
            let x = self.v[i].x;
            if x<mini{
                mini = x;
            }
        }
        mini
    }

    fn max_y(&self)->i32{
        let mut maxi = self.v[0].y + self.y;
        for i in 1..4{
            let y = self.v[i].y + self.y;
            if y>maxi{
                maxi = y;
            }
        }
        maxi
    }

    fn max_y_v(&self)->i32{
        let mut maxi = self.v[0].y;
        for i in 1..4{
            let y = self.v[i].y;
            if y>maxi{
                maxi = y;
            }
        }
        maxi
    }

    fn column(&self)->i32{
        return self.x / CELL_SIZE;
    }

}

#[derive(PartialEq, Copy, Clone)]
pub enum GameMode {
    StandBy,
    Play,
    GameOver,
    HightScore,
    HallOfFame,
}

#[derive(Clone)]
struct HightScore{
    name : String,
    score : i32,
}

struct Game {
    board: [i32; (NB_ROWS * NB_COLUMNS) as usize],
    cur_score: i32,
    mode : GameMode,
    cur_shape : TetrisShape,
    next_shape : TetrisShape,
    velo_h : i32,
    f_drop : bool,
    f_fast_down : bool,
    hight_scores : Vec<HightScore>,
    player_name: String,
    id_hight_score : Option<usize>,
    i_hight_score_color : i32,
    process_event : fn ( self1 : &mut Game, event : &Event)->bool,
    f_play_sound : bool,
    ascii_table : HashMap<sdl2::keyboard::Keycode,char>,
    horizontal_move : i32,
    horizontal_start_column : i32,
}

impl Game{
    fn new() -> Self {

        Self {
            board: [0; (NB_ROWS * NB_COLUMNS) as usize],
            cur_score: 0,
            mode : GameMode::StandBy,
            cur_shape : TetrisShape::default(),
            next_shape : TetrisShape::default(),
            velo_h : 0,
            f_drop : false,
            f_fast_down : false,
            hight_scores : Vec::new(),
            player_name : String::with_capacity(10),
            id_hight_score : None,
            i_hight_score_color : 0,
            process_event : Game::process_standby_event,
            f_play_sound : false,
            ascii_table : HashMap::from([
                (Keycode::A, 'A'),
                (Keycode::B, 'B'),
                (Keycode::C, 'C'),
                (Keycode::D, 'D'),
                (Keycode::E, 'E'),
                (Keycode::F, 'F'),
                (Keycode::G, 'G'),
                (Keycode::H, 'H'),
                (Keycode::I, 'I'),
                (Keycode::J, 'J'),
                (Keycode::K, 'K'),
                (Keycode::L, 'L'),
                (Keycode::M, 'M'),
                (Keycode::N, 'N'),
                (Keycode::O, 'O'),
                (Keycode::P, 'P'),
                (Keycode::Q, 'Q'),
                (Keycode::R, 'R'),
                (Keycode::S, 'S'),
                (Keycode::T, 'T'),
                (Keycode::U, 'U'),
                (Keycode::V, 'V'),
                (Keycode::W, 'W'),
                (Keycode::X, 'X'),
                (Keycode::Y, 'Y'),
                (Keycode::Z, 'Z'),
                (Keycode::Kp0, '0'),
                (Keycode::Kp1, '1'),
                (Keycode::Kp2, '2'),
                (Keycode::Kp3, '3'),
                (Keycode::Kp4, '4'),
                (Keycode::Kp5, '5'),
                (Keycode::Kp6, '6'),
                (Keycode::Kp7, '7'),
                (Keycode::Kp8, '8'),
                (Keycode::Kp9, '9'),
                (Keycode::Num0, '0'),
                (Keycode::Num1, '1'),
                (Keycode::Num2, '2'),
                (Keycode::Num3, '3'),
                (Keycode::Num4, '4'),
                (Keycode::Num5, '5'),
                (Keycode::Num6, '6'),
                (Keycode::Num7, '7'),
                (Keycode::Num8, '8'),
                (Keycode::Num9, '9'),
            ]),
            horizontal_move : 0,
            horizontal_start_column : -1,
        }
    }

    fn init_board(&mut self) {
        //--
        for y in 0..NB_ROWS {
            for x in 0..NB_COLUMNS {
                self.board[(x + y * NB_COLUMNS) as usize] = 0;
            }
        }
    }

    fn is_over(&self) -> bool {
        //--
        for x in 0..NB_COLUMNS {
            if self.board[x as usize] != 0 {
                return true;
            }
        }
        false
    }

    fn erase_completed_lines(&mut self) -> i32 {
        //--------------------------------------------------------
        let mut nbL = 0;
        for y in 0..NB_ROWS {
            //-- Check completed line
            let mut f_complete = true;
            for x in 0..NB_COLUMNS {
                if self.board[(x + y * NB_COLUMNS) as usize] == 0 {
                    f_complete = false;
                    break;
                }
            }
            if f_complete {
                nbL += 1;
                //-- Shift down the game board
                let mut y1 = y;
                while y1 > 0 {
                    let ySrcOffset = (y1 - 1) * NB_COLUMNS;
                    let yDesOffset = y1 * NB_COLUMNS;
                    for x in 0..NB_COLUMNS {
                        self.board[(x + yDesOffset) as usize] =
                            self.board[(x + ySrcOffset) as usize]
                    }
                    y1 -= 1;
                }
            }
        }
        // return number of erase lines
        nbL
    }

    fn frezze_tetromino(&mut self) -> bool {
        let mut x: i32;
        let mut y: i32;
        let mut ix: i32;
        let mut iy: i32;

        ix = (self.cur_shape.x + 1) / CELL_SIZE;
        iy = (self.cur_shape.y + 1) / CELL_SIZE;
        for i in 0..4 {
            x = self.cur_shape.v[i].x + ix;
            y = self.cur_shape.v[i].y + iy;
            if ((x>=0) && (x<NB_COLUMNS) && (y>=0) && (y<NB_ROWS)){
                self.board[(x + y * NB_COLUMNS) as usize] = self.cur_shape.typ;
            }
        }

        let nb_lines = self.erase_completed_lines();
        if nb_lines > 0 {
            self.cur_score += compute_score(nb_lines);
            self.f_play_sound = true;
            // if let Some(ref mut succes_sound) = self.success_sound {
            //     succes_sound.set_volume(10.0);
            //     succes_sound.play();
            // }
            return true;
        }
        false
    }

    fn new_tetromino(&mut self){
        self.cur_shape.init(5*CELL_SIZE, 0, self.next_shape.typ);
        self.cur_shape.y = -(self.cur_shape.max_y()+1)*CELL_SIZE;
        self.next_shape.init((NB_COLUMNS + 3)*CELL_SIZE, (NB_ROWS / 2)*CELL_SIZE, (rand::random::<u8>() % 6 + 1) as i32);
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) {
        let mut l: i32;
        let mut t: i32;

        canvas.set_draw_color(Color{r:20,g:20,b:100,a:255});
        canvas.fill_rect(Rect::new(LEFT as i32,TOP as i32,
                    (NB_COLUMNS*CELL_SIZE) as u32,(NB_ROWS*CELL_SIZE) as u32));

        let a = (CELL_SIZE - 2) as u32;
        for x in 0..NB_COLUMNS {
            for y in 0..NB_ROWS {
                let typ = self.board[(x + y * NB_COLUMNS) as usize];

                if typ != 0 {
                    let c = TETRIS_COLORS[typ as usize];
                    l = (x * (CELL_SIZE) + LEFT + 1);
                    t = (y * (CELL_SIZE) + TOP + 1);
                    canvas.set_draw_color(c);
                    canvas.fill_rect(Rect::new(l as i32,t as i32,a,a));
            
                }
            }
        }

        if self.mode==GameMode::Play {
            self.cur_shape.draw(canvas);
        }

        self.next_shape.draw(canvas);


    }

    fn load_hight_scores(&mut self){
        //--
        self.hight_scores = Vec::new();
        // self.hight_scores.push(Hight_Score{name:"azeret1".to_string(),score:100});
        // self.hight_scores.push(Hight_Score{name:"azeret2".to_string(),score:200});
        // self.hight_scores.push(Hight_Score{name:"azeret3".to_string(),score:300});

        let path = Path::new("high_scores.txt");
        let display = path.display();
        let _ = match File::open(&path){
            Err(why) => panic!("Couldn't open {}: {}", display, why),
            Ok(file) => {
                let f = io::BufReader::new(file);
                for line in f.lines(){
                    if let Ok(str_line) = line {
                        let split : Vec<&str> = str_line.split(",").collect();
                        let name = split[0].to_string();
                        let score = split[1].trim().parse::<i32>().unwrap();
                        self.hight_scores.push(HightScore{name,score});
                    }
                }
            }
        };

    }

    fn save_hight_scores(&mut self){
        //--
        let path = Path::new("high_scores.txt");
        let display = path.display();
        let _ = match File::create(&path){
            Err(why) => panic!("Couldn't create {}: {}", display, why),
            Ok(file) => {
                let mut file = LineWriter::new(file);
                for h in self.hight_scores.iter() {
                    let val = format!("{},{}\n",h.name,h.score);
                    file.write_all(&val.as_bytes());
                }
            }
        };
    }

    fn is_hight_score(&mut self)->Option<usize>{
        let mut i : usize = 0;
        for h in self.hight_scores.iter() {
            if self.cur_score>h.score {
                self.id_hight_score = Some(i);
                return Some(i); 
            }
            i+=1;
        }
        self.id_hight_score = None;
        None
    }

    fn set_hight_score_name(&mut self, iscore : usize){
        self.hight_scores[iscore].name = self.player_name.clone();
    }

    fn set_hight_score_value(&mut self, iscore : usize){
        self.hight_scores[iscore].score = self.cur_score;
    }

    fn insert_hight_score(&mut self, iscore : usize, new_hight : HightScore){
        self.hight_scores.insert(iscore, new_hight);
        self.hight_scores.pop();
    }
    
    fn draw_score(&mut self,canvas: &mut WindowCanvas, font : &Font){

        let texture_creator = canvas.texture_creator();

        //--
        let score_string = format!("SCORE : {:06}",self.cur_score);
        let surface = font
            .render(&score_string)
            .blended(Color::RGB(255, 0, 0))
            .map_err(|e| e.to_string()).unwrap();
            
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let target_rect = Rect::new(0,(TOP+(NB_ROWS+1)*CELL_SIZE) as i32, w, h);
        canvas.copy(&texture, None, target_rect);

    }

    fn draw_greeting(&mut self,canvas: &mut WindowCanvas, font : &Font){

        let texture_creator = canvas.texture_creator();

        //--
        let score_string = format!("TETRIS in SDL2");
        let surface = font
            .render(&score_string)
            .blended(Color::RGB(255, 255, 0))
            .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let mut y = (TOP+(NB_ROWS/2-2)*CELL_SIZE) as i32;
        let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
        canvas.copy(&texture, None, target_rect);
    
        y += 6 + h as i32;
        let score_string = format!("by");
        let surface = font
        .render(&score_string)
        .blended(Color::RGB(255, 255, 0))
        .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();
            let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
            canvas.copy(&texture, None, target_rect);
    
        y += 6 + h as i32;
        let score_string = format!("Raymond NGUYEN");
        let surface = font
        .render(&score_string)
        .blended(Color::RGB(255, 255, 0))
        .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();
            let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
            canvas.copy(&texture, None, target_rect);
    
    }


    fn process_standby_event(&mut self, event : &Event)->bool{
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape),..} => 
            { 
                return true
            }
            Event::KeyDown { keycode: Some(Keycode::Space),..} =>
            {
                self.mode = GameMode::Play;
                self.new_tetromino();
                self.init_board();
                self.process_event = Game::process_play_event;
            }
             _ => {}
        }
        false
    }

    fn process_play_event(&mut self, event : &Event)->bool{

        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape),..} => { 
                return true
            }

            Event::KeyDown { keycode: Some(Keycode::Left),..} => 
            {
                self.velo_h = -1;
            }
            Event::KeyDown { keycode: Some(Keycode::Right),..} => {
                self.velo_h = 1;
            }
            Event::KeyDown { keycode: Some(Keycode::Up),..} => 
            {
                self.cur_shape.rotate_left();

                if  self.cur_shape.hit_ground1(&self.board) {
                    self.cur_shape.rotate_right();
                }else if self.cur_shape.is_out_right(){
                    let backupX = self.cur_shape.x;
                    //-- Move Tetromino in board
                    while self.cur_shape.is_out_right() {
                        self.cur_shape.x -= 1;
                    }
                    if (self.cur_shape.hit_ground1(&self.board)){
                        //-- Undo
                        self.cur_shape.x = backupX;
                        self.cur_shape.rotate_right();
                    }
                }else if self.cur_shape.is_out_left(){
                    let backupX = self.cur_shape.x;
                    //-- Move Tetromino in board
                    while self.cur_shape.is_out_left() {
                        self.cur_shape.x += 1;
                    }
                    if (self.cur_shape.hit_ground1(&self.board)){
                        //-- Undo
                        self.cur_shape.x = backupX;
                        self.cur_shape.rotate_right();
                    }
                }

            }
            Event::KeyDown { keycode: Some(Keycode::Down),..} =>
            {
                self.f_fast_down = true;
                // self.cur_shape.rotate_right();
                // if !self.cur_shape.is_in_board() || self.cur_shape.hit_ground(&self.board) {
                //     self.cur_shape.rotate_left();
                // }
            }
            Event::KeyDown { keycode: Some(keycode),..} =>
            {
                if *keycode == Keycode::Space {
                    self.f_drop = true;
                }
            }
            Event::KeyUp { keycode: Some(Keycode::Down),..}=> {
                //println!("Released Key ");
                self.f_fast_down = false;
            }
            Event::KeyUp { keycode: Some(Keycode::Left),..} => {
                //println!("Released Key ");
                self.velo_h = 0;
            }
            Event::KeyUp { keycode: Some(Keycode::Right),..} => {
                //println!("Released Key ");
                self.velo_h = 0;
            }
            _ => {}
        }

        false

    }


    fn draw_game_over (&mut self,canvas: &mut WindowCanvas, font : &Font){

        let texture_creator = canvas.texture_creator();

        //--
        let score_string = format!("GAME OVER");
        let surface = font
            .render(&score_string)
            .blended(Color::RGB(255, 0, 0))
            .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let mut y = (TOP+(NB_ROWS/2-2)*CELL_SIZE) as i32;
        let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
        canvas.copy(&texture, None, target_rect);
    
        y += 2*h as i32;
        let score_string = format!("Press SPACE to Play again");
        let surface = font
        .render(&score_string)
        .blended(Color::RGB(255, 255, 0))
        .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();
            let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
            canvas.copy(&texture, None, target_rect);


    }

    fn process_game_over_event(&mut self, event : &Event)->bool{
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape),..} => { 
                return true
            }
            Event::KeyDown {keycode: Some(Keycode::Space),..} =>
            {
                self.mode = GameMode::Play;
                self.new_tetromino();
                self.init_board();
                self.cur_score = 0;
                self.process_event = Game::process_play_event;
            }
            _ => {}
        }
        false
    }

    fn process_hight_scores_event(&mut self, event : &Event)->bool{

        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape),..} => { 
                return true
            }
            Event::KeyDown {keycode: Some(Keycode::Backspace),..} =>
            {
                if self.player_name.len()>0{
                    self.player_name.pop();
                    if let Some(iscore) = self.id_hight_score {
                        self.set_hight_score_name(iscore);
                    }
                }
            }
            Event::KeyDown {keycode: Some(Keycode::KpEnter),..} | Event::KeyDown {keycode: Some(Keycode::Return),..} => 
            {
                if let Some(iscore) = self.id_hight_score {
                    if self.player_name.len()==0{
                        self.player_name = format!("XXXX");
                    }
                    self.set_hight_score_name(iscore);
                }
                self.save_hight_scores();
                self.mode=GameMode::StandBy;
                self.process_event = Game::process_standby_event;

            }
            Event::KeyDown {keycode: Some(code),..} =>
            {
                let icode = *code as i32;
                if (icode>=(Keycode::A as i32) && icode<=(Keycode::Z as i32)) ||
                    (icode>=(Keycode::Kp0 as i32) && icode<=(Keycode::Kp9 as i32))||
                    (icode>=(Keycode::Num0 as i32) && icode<=(Keycode::Num9 as i32)) {

                }
                let c0 = self.ascii_table.get(code);
                if let Some(c) = c0{
                    self.player_name.push(c.clone());
                    //println!(">>{}",self.player_name);
                    if let Some(iscore) = self.id_hight_score {
                        self.set_hight_score_name(iscore);
                    }
                }
            }
             _ => {}
        }

        false

    }   

    fn draw_hight_scores(&mut self,canvas: &mut WindowCanvas, font : &Font) {

        let texture_creator = canvas.texture_creator();
        
        let x_col0 = (LEFT + CELL_SIZE) as i32;
        let x_col1 = (LEFT + 7*CELL_SIZE) as i32;

        //--
        let score_string = format!("HIGHT SCORES");
        let surface = font
            .render(&score_string)
            .blended(Color::RGB(255, 0, 0))
            .map_err(|e| e.to_string()).unwrap();
        let (w, h) = surface.size();
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();

        let mut y = (TOP+CELL_SIZE) as i32;
        let target_rect = Rect::new((LEFT+(NB_COLUMNS*CELL_SIZE-(w as i32))/2) as i32, y, w, h);
        canvas.copy(&texture, None, target_rect);

        y += 3 * h as i32;

        for i in 0..9 {

            let mut c = Color::RGB(255, 255, 0);
            if let Some(iscore) = self.id_hight_score {
                if i == iscore {
                    c = if (self.i_hight_score_color % 2)==0 {
                            Color::RGB(0, 0, 255)
                        }else{
                            Color::RGB(255, 255, 255)
                        };
                }
            }

            let hs = &self.hight_scores[i];
            if hs.name.len()>0 {
                let surface = font
                .render(&hs.name)
                .blended(c)
                .map_err(|e| e.to_string()).unwrap();
                let (w, h) = surface.size();
                let texture = texture_creator.create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string()).unwrap();
                let target_rect = Rect::new(x_col0, y, w, h);
                canvas.copy(&texture, None, target_rect);
            }
            let score = format!("{:06}",hs.score);
            let surface = font
            .render(&score)
            .blended(c)
            .map_err(|e| e.to_string()).unwrap();
            let (w, h) = surface.size();
            let texture = texture_creator.create_texture_from_surface(&surface)
                .map_err(|e| e.to_string()).unwrap();
            let target_rect = Rect::new(x_col1, y, w, h);
            canvas.copy(&texture, None, target_rect);

            y += h as i32 + 2;


        }

   }

    
}

fn compute_score(nb_lines: i32) -> i32 {
    match nb_lines {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => 2000,
    }
}

pub fn main() {

    let mut music_playing : bool = false;

    let sdl_context = sdl2::init().expect("SDL initialization failed");
    let video_subsystem = sdl_context.video().unwrap();
    //let audio_subsystem = sdl_context.audio().unwrap();

    let window = video_subsystem.window("SDL Tetris",WIN_WIDTH as u32,WIN_HEIGHT as u32)
    .position_centered()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
    let _mixer_context =
        sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG ).unwrap();

    // Number of mixing channels available for sound effect `Chunk`s to play
    // simultaneously.
    sdl2::mixer::allocate_channels(4);

    {
        let n = sdl2::mixer::get_chunk_decoders_number();
        println!("available chunk(sample) decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_chunk_decoder(i));
        }
    }

    {
        let n = sdl2::mixer::get_music_decoders_number();
        println!("available music decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_music_decoder(i));
        }
    }

    let mut full_path = std::env::current_dir().unwrap();
    full_path.push("Tetris.wav");    
    let music = sdl2::mixer::Music::from_file(full_path).unwrap();

    // fn hook_finished() {
    //     println!("play ends! from rust cb");
    // }

    // sdl2::mixer::Music::hook_finished(||{
    //     println!("play ends! from rust cb");
    //     //music_playing = false;

    // });

    sdl2::mixer::Music::set_volume(16);

    println!("music => {:?}", music);
    println!("music type => {:?}", music.get_type());
    println!("music volume => {:?}", sdl2::mixer::Music::get_volume());
    println!("play => {:?}", music.play(-1));
    music_playing = true;


    let mut sound_path = std::env::current_dir().unwrap();
    sound_path.push("109662__grunz__success.wav");    
    let mut sound_chunk = sdl2::mixer::Chunk::from_file(sound_path)
            .map_err(|e| format!("Cannot load sound file: {:?}", e))
            .unwrap();
    sound_chunk.set_volume(16);
    //sdl2::mixer::Channel::all().play(&sound_chunk, 0).unwrap();

    let texture_creator = canvas.texture_creator();

    //-- 
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let mut font_path = std::env::current_dir().unwrap();
    font_path.push("sansation.ttf");
    let mut font18 = ttf_context.load_font(&font_path,18).unwrap();
    font18.set_style(sdl2::ttf::FontStyle::BOLD|sdl2::ttf::FontStyle::ITALIC);

    //font_path = std::env::current_dir().unwrap();
    //font_path.push("sansation.ttf");
    let mut font20 = ttf_context.load_font(&font_path,20).unwrap();
    font20.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut game = Game::new();
    game.next_shape.init((NB_COLUMNS + 3)*CELL_SIZE, (NB_ROWS / 2)*CELL_SIZE, (rand::random::<u8>() % 6 + 1) as i32);
    game.load_hight_scores();
    game.mode = GameMode::StandBy;

    let mut update_timer_v = Instant::now();
    let mut update_timer_h = Instant::now();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;


    'running: loop {
        
        //i = (i+ 1) % 255;
        canvas.set_draw_color(Color::RGB(30,30,80));
        canvas.clear();

        for event in event_pump.poll_iter(){

            if (game.process_event) (&mut game,&event) {
                if let Some(i)=game.is_hight_score(){
                    game.insert_hight_score(i , HightScore{name:game.player_name.clone(),score:game.cur_score});
                    game.cur_score = 0;
                    game.mode=GameMode::HightScore;
                    game.process_event = Game::process_hight_scores_event;
                }else{
                    break 'running;
                }
            }

        }

        //-- Update Game State
        if game.mode==GameMode::Play {

            if  {

            }else if game.horizontal_move!=0 {

                let elapsed = update_timer_h.elapsed().as_millis();
                if elapsed > 20 {
                    update_timer_h = Instant::now();

                    for _i in 0..4 {
                        let backupX = game.cur_shape.x;
                        game.cur_shape.x += game.horizontal_move;

                        if game.horizontal_move<0 {
                            if game.cur_shape.is_out_left() {
                                game.cur_shape.x = backupX;
                                game.horizontal_move = 0;
                                break;
                            } else if game.cur_shape.hit_ground1(&game.board) {
                                game.cur_shape.x = backupX;
                                game.horizontal_move = 0;
                                break;
                            }

                        }else if game.horizontal_move>0 {
                            if game.cur_shape.is_out_right() {
                                game.cur_shape.x = backupX;
                                game.horizontal_move = 0;
                                break;
                            } else if game.cur_shape.hit_ground1(&game.board) {
                                game.cur_shape.x = backupX;
                                game.horizontal_move = 0;
                                break;
                            }

                        }

                        if game.horizontal_move!=0 {
                            if game.horizontal_start_column!=game.cur_shape.column() {
                                game.cur_shape.x = backupX;
                                game.horizontal_move = 0;
                                break;
                            }

                        }

                    }

                }

            }else if game.f_drop {

                let elapsed = update_timer_v.elapsed().as_millis();
                if elapsed > 10 {
                    update_timer_v = Instant::now();

                    for _i in 0..6 {
                        //-- Move down for checking 
                        game.cur_shape.y += 1;
                        if game.cur_shape.hit_ground1(&game.board){
                            game.cur_shape.y -= 1;
                            game.frezze_tetromino();
                            game.new_tetromino();
                            game.f_drop = false;
                        }else if game.cur_shape.is_out_bottom() {
                            game.cur_shape.y -= 1;
                            game.frezze_tetromino();
                            game.new_tetromino();
                            game.f_drop = false;

                        }

                        if  game.f_drop {
                            if game.velo_h!=0 {
                                let elapsed = update_timer_h.elapsed().as_millis();
                                if elapsed>20 {
                                    let backupX = game.cur_shape.x;
                                    game.cur_shape.x += game.velo_h;
                                    if game.velo_h<0 {
                                        if game.cur_shape.is_out_left() {
                                            game.cur_shape.x = backupX;
                                        }else{
                                            update_timer_h = Instant::now();
                                            game.horizontal_move = game.velo_h;
                                            game.horizontal_start_column = game.cur_shape.column();
                                            break;
                                        }

                                    }else if game.velo_h>0 {
                                        if game.cur_shape.is_out_right() {
                                            game.cur_shape.x = backupX;
                                        }else{
                                            update_timer_h = Instant::now();
                                            game.horizontal_move = game.velo_h;
                                            game.horizontal_start_column = game.cur_shape.column();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                
                }

            } else {

                let elapsed = update_timer_v.elapsed().as_millis();
                let limit = if game.f_fast_down { 10 }else{ 20 };
                if elapsed > limit {
                    update_timer_v = Instant::now();

                    for i in 0..3 {
                        //-- Move down to Check
                        game.cur_shape.y += 1;
                        let mut fMove : bool = true;
                        if game.cur_shape.hit_ground1(&game.board) {
                            game.cur_shape.y -= 1;
                            game.frezze_tetromino();
                            game.new_tetromino();
                            fMove = false;
                        }else if (game.cur_shape.is_out_bottom()){
                            game.cur_shape.y -= 1;
                            game.frezze_tetromino();
                            game.new_tetromino();
                            fMove = false;
                        }

                        if (fMove){

                            let elapsed = update_timer_h.elapsed().as_millis();
                            if game.velo_h!=0 {

                                if (elapsed>15){
                                    update_timer_h = Instant::now();
                                    game.cur_shape.x += game.velo_h;
                                    if game.velo_h<0 {
                                        if (game.cur_shape.is_out_left()){
                                            game.cur_shape.x -= game.velo_h;
                                        }else{
                                            if game.cur_shape.hit_ground1(&game.board) {
                                                game.cur_shape.x -= game.velo_h;
                                            }else{
                                                game.horizontal_move = game.velo_h;
                                                game.horizontal_start_column = game.cur_shape.column();
                                                break;
                                            }
                                        }
                                    }else if game.velo_h>0 {
                                        if (game.cur_shape.is_out_right()){
                                            game.cur_shape.x -= game.velo_h;
                                        }else{
                                            if game.cur_shape.hit_ground1(&game.board) {
                                                game.cur_shape.x -= game.velo_h;
                                            }else{
                                                game.horizontal_move = game.velo_h;
                                                game.horizontal_start_column = game.cur_shape.column();
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            //-- Check Game Over
            if game.is_over(){
                game.init_board();
                if let Some(i)=game.is_hight_score(){
                    game.insert_hight_score(i , HightScore{name:game.player_name.clone(),score:game.cur_score});
                    game.cur_score = 0;
                    game.mode=GameMode::HightScore;
                    game.process_event = Game::process_hight_scores_event;
                }else{
                    game.mode = GameMode::GameOver;
                    game.process_event = Game::process_game_over_event;
                } 
            }
               
        }else if game.mode==GameMode::HightScore{
            let elapsed = update_timer_v.elapsed().as_millis();
            if elapsed > 300 {
                update_timer_v = Instant::now();
                game.i_hight_score_color += 1;
            }
        }

        //-- Play Sound
        if game.f_play_sound {
            game.f_play_sound = false;
            sdl2::mixer::Channel::all().play(&sound_chunk, 0).unwrap();
        }

        //canvas.set_draw_color(Color::RGB(30,0,0));
        //canvas.draw_line(Point::new(10,10), Point::new(100,100));

        game.draw(&mut canvas);

        match game.mode {
            GameMode::StandBy =>{
                game.draw_greeting(&mut canvas, &font20);
            },
            GameMode::GameOver => {
                game.draw_game_over(&mut canvas, &font20);
            },
            GameMode::HightScore => {
                game.draw_hight_scores(&mut canvas, &font18);
            },
            _ => {}
            
        }

        game.draw_score(&mut canvas, &font18);

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000u32 / 60));

    }
    
    sdl2::mixer::Music::halt();

}