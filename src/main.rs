use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;

const MAP_HEIGHT: usize = 60;
const MAP_WIDTH: usize = 80;
const TILE_SIZE: f32 = 10.0;

//double buffering the grid so that we dont need to clone the array
//between the current and next state transitions.
const BUFFER_COUNT: usize = 2;
type Grid = Vec<Vec<Vec<i32>>>;

struct MainState {
    buffers: Grid,
    active_buffer: usize,
    seconds_since_update: f32,
    simulating: bool
}

impl MainState {
    fn new() -> ggez::GameResult<MainState> {

        let mut buffers =  vec![vec![vec![0; MAP_HEIGHT]; MAP_WIDTH]; BUFFER_COUNT];
    
        let active_buffer: usize = 0;

        buffers[active_buffer][5][5] = 1;
        buffers[active_buffer][5][6] = 1;
        buffers[active_buffer][5][7] = 1;

        buffers[active_buffer][6][6] = 1;
        buffers[active_buffer][6][7] = 1;
        buffers[active_buffer][6][8] = 1;

        let state = MainState {
            buffers: buffers,
            active_buffer: active_buffer,
            seconds_since_update: 0.0,
            simulating: true
        };
        
        Ok(state)
    }

    //checks to see if the cell in the active buffer is alive and returns a 1 or a 0
    fn is_cell_alive_i32(&self, x: usize, y: usize) -> i32 {
        return if x < MAP_WIDTH && y < MAP_HEIGHT && self.buffers[self.active_buffer][x][y] == 1
         { 1 } else { 0 };
    }
    
    //checks to see if the cell in the active buffer is alive
    fn is_cell_alive(&self, x: usize, y: usize) -> bool{
        return x < MAP_WIDTH && y < MAP_HEIGHT && self.buffers[self.active_buffer][x][y] == 1;
    }

    //determines the number of live neighbors around a cell
    fn number_of_neighbors(&self, x: usize, y: usize) -> i32 {

        let mut count: i32 = 0;
        
        if x < MAP_WIDTH {

            if y >= 1 && y < MAP_HEIGHT {
                count += self.is_cell_alive_i32(x, y - 1);
            }

            if y < MAP_HEIGHT - 1 {
                count += self.is_cell_alive_i32(x,  y + 1);
            }

        }

        if x < MAP_WIDTH - 1 {

            if y >= 1 && y < MAP_HEIGHT{
                count += self.is_cell_alive_i32(x + 1, y - 1);
            }

            if y < MAP_HEIGHT{
                count += self.is_cell_alive_i32(x + 1, y);
            }

            if y < MAP_HEIGHT - 1{
                count += self.is_cell_alive_i32(x + 1, y + 1);
            }
        }


        if x >= 1 && x < MAP_WIDTH {
            
            if y < (MAP_HEIGHT - 1) {
                count += self.is_cell_alive_i32(x - 1,  y + 1);
            }

            if y < MAP_HEIGHT {
                count += self.is_cell_alive_i32(x - 1,  y);
            }

            if y >= 1 && y < MAP_HEIGHT {
                count += self.is_cell_alive_i32(x - 1,  y - 1);
            }
            
        }

        (count)
    }   

    //returns the buffer that is not currently active
    fn get_secondary_buffer_index(&self) -> usize {
        return if self.active_buffer == 1 { ( 0 ) } else { ( 1 ) }
    }

}

impl event::EventHandler for MainState {
 
    fn mouse_button_down_event(&mut self, _ctx: &mut ggez::Context, button: ggez::input::mouse::MouseButton, x: f32, y: f32) {
        
        if button == ggez::input::mouse::MouseButton::Left {
            let cell_x: usize = (x / TILE_SIZE).floor() as usize;
            let cell_y: usize = (y / TILE_SIZE).floor() as usize;

            if cell_x < MAP_WIDTH && cell_y < MAP_HEIGHT {
                self.buffers[self.active_buffer][cell_x][cell_y] = 1;
            }
        } 
        else if button == ggez::input::mouse::MouseButton::Right {
            self.simulating = !self.simulating;
        }        
    }
 
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        
        let dt = ggez::timer::delta(_ctx);

        self.seconds_since_update += dt.as_secs_f32();

        if self.seconds_since_update >= 1.0 && self.simulating {
            self.seconds_since_update = 0.0;
            
            let secondary: usize = self.get_secondary_buffer_index();

            for x in 0..MAP_WIDTH {
                for y in 0..MAP_HEIGHT {

                    let count: i32 = self.number_of_neighbors(x, y);
                    let alive: bool = self.is_cell_alive(x, y);
  
                    if alive && count < 2 {
                        self.buffers[secondary][x][y] = 0;               
                    } else if alive && (count == 2 || count == 3) {
                        self.buffers[secondary][x][y] = 1;  
                    }  else if alive && count > 3 {
                        self.buffers[secondary][x][y] = 0;  
                    } else if !alive && count == 3 {
                        self.buffers[secondary][x][y] = 1;  
                    } else {
                        self.buffers[secondary][x][y] = self.buffers[self.active_buffer][x][y];  
                    }
                }
            }

            self.active_buffer = secondary;
            
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());


        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let f32_x = x as f32 * TILE_SIZE;
                let f32_y = y as f32 * TILE_SIZE;


                if self.buffers[self.active_buffer][x][y] == 1 {
                        
                        let rectangle = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            graphics::Rect::new(0.0, 0.0, TILE_SIZE, TILE_SIZE),
                            graphics::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                        )?;

                        graphics::draw(
                            ctx,
                            &rectangle,
                            (na::Point2::new(
                            f32_x,
                                f32_y,
                            ),),
                        )?;
                }            
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("rust-ggez-conway", "David");
    let (ctx, event_loop) = &mut cb
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
        .build()?;
    let state = &mut MainState::new()?;
    event::run(ctx, event_loop, state)
}