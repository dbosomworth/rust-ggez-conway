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

// Memory-optimized grid structure:
// Instead of Vec<Vec<Vec<i32>>>, we use a single flat Vec<i32>
// This reduces memory overhead and improves cache locality
// Layout: [buffer0_data, buffer1_data]
// where each buffer_data is laid out as: [row0, row1, ..., row59]
// and each row contains MAP_WIDTH cells
// Total size: BUFFER_COUNT * MAP_WIDTH * MAP_HEIGHT = 2 * 80 * 60 = 9600 elements
type Grid = Vec<i32>;

struct MainState {
    buffers: Grid,
    active_buffer: usize,
    seconds_since_update: f32,
    simulating: bool
}

impl MainState {
    fn new() -> ggez::GameResult<MainState> {
        // Initialize flat vector with all cells dead (0)
        // Size: BUFFER_COUNT * MAP_WIDTH * MAP_HEIGHT
        let mut buffers = vec![0; BUFFER_COUNT * MAP_WIDTH * MAP_HEIGHT];
    
        let active_buffer: usize = 0;

        // Set initial pattern (glider gun seed pattern)
        // Using the new flat structure with helper method
        let initial_cells = [(5, 5), (5, 6), (5, 7), (6, 6), (6, 7), (6, 8)];
        for (x, y) in initial_cells.iter() {
            let index = MainState::calculate_index(active_buffer, *x, *y);
            buffers[index] = 1;
        }

        let state = MainState {
            buffers: buffers,
            active_buffer: active_buffer,
            seconds_since_update: 0.0,
            simulating: true
        };
        
        Ok(state)
    }

    /// Calculates the flat array index from 3D coordinates (buffer, x, y)
    /// Formula: buffer * (MAP_WIDTH * MAP_HEIGHT) + y * MAP_WIDTH + x
    /// This ensures proper memory layout for cache efficiency
    fn calculate_index(buffer: usize, x: usize, y: usize) -> usize {
        buffer * (MAP_WIDTH * MAP_HEIGHT) + y * MAP_WIDTH + x
    }

    /// Gets the cell value at the specified coordinates in the given buffer
    fn get_cell(&self, buffer: usize, x: usize, y: usize) -> i32 {
        if x < MAP_WIDTH && y < MAP_HEIGHT {
            let index = MainState::calculate_index(buffer, x, y);
            self.buffers[index]
        } else {
            0
        }
    }

    /// Sets the cell value at the specified coordinates in the given buffer
    fn set_cell(&mut self, buffer: usize, x: usize, y: usize, value: i32) {
        if x < MAP_WIDTH && y < MAP_HEIGHT {
            let index = MainState::calculate_index(buffer, x, y);
            self.buffers[index] = value;
        }
    }

    //checks to see if the cell in the active buffer is alive and returns a 1 or a 0
    fn is_cell_alive_i32(&self, x: usize, y: usize) -> i32 {
        return if x < MAP_WIDTH && y < MAP_HEIGHT && self.get_cell(self.active_buffer, x, y) == 1
         { 1 } else { 0 };
    }
    
    //checks to see if the cell in the active buffer is alive
    fn is_cell_alive(&self, x: usize, y: usize) -> bool{
        return x < MAP_WIDTH && y < MAP_HEIGHT && self.get_cell(self.active_buffer, x, y) == 1;
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
                self.set_cell(self.active_buffer, cell_x, cell_y, 1);
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
                        self.set_cell(secondary, x, y, 0);               
                    } else if alive && (count == 2 || count == 3) {
                        self.set_cell(secondary, x, y, 1);  
                    }  else if alive && count > 3 {
                        self.set_cell(secondary, x, y, 0);  
                    } else if !alive && count == 3 {
                        self.set_cell(secondary, x, y, 1);  
                    } else {
                        let current_value = self.get_cell(self.active_buffer, x, y);
                        self.set_cell(secondary, x, y, current_value);  
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


                if self.get_cell(self.active_buffer, x, y) == 1 {
                        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_index() {
        // Test buffer 0, position (0, 0) should be at index 0
        assert_eq!(MainState::calculate_index(0, 0, 0), 0);
        
        // Test buffer 0, position (1, 0) should be at index 1
        assert_eq!(MainState::calculate_index(0, 1, 0), 1);
        
        // Test buffer 0, position (0, 1) should be at index MAP_WIDTH (80)
        assert_eq!(MainState::calculate_index(0, 0, 1), MAP_WIDTH);
        
        // Test buffer 1, position (0, 0) should be at the start of second buffer
        assert_eq!(MainState::calculate_index(1, 0, 0), MAP_WIDTH * MAP_HEIGHT);
        
        // Test arbitrary position in buffer 0
        let x = 10;
        let y = 5;
        let expected = y * MAP_WIDTH + x;
        assert_eq!(MainState::calculate_index(0, x, y), expected);
        
        // Test arbitrary position in buffer 1
        let expected_buffer1 = MAP_WIDTH * MAP_HEIGHT + y * MAP_WIDTH + x;
        assert_eq!(MainState::calculate_index(1, x, y), expected_buffer1);
    }

    #[test]
    fn test_get_set_cell() {
        let mut state = MainState::new().unwrap();
        
        // Test setting and getting a cell in buffer 0
        state.set_cell(0, 10, 10, 1);
        assert_eq!(state.get_cell(0, 10, 10), 1);
        
        // Test that other cells are still 0
        assert_eq!(state.get_cell(0, 11, 10), 0);
        assert_eq!(state.get_cell(0, 10, 11), 0);
        
        // Test setting and getting a cell in buffer 1
        state.set_cell(1, 20, 20, 1);
        assert_eq!(state.get_cell(1, 20, 20), 1);
        
        // Test that buffer 0 is not affected
        assert_eq!(state.get_cell(0, 20, 20), 0);
        
        // Test bounds checking - out of bounds should return 0
        assert_eq!(state.get_cell(0, MAP_WIDTH, 0), 0);
        assert_eq!(state.get_cell(0, 0, MAP_HEIGHT), 0);
    }

    #[test]
    fn test_initial_pattern() {
        let state = MainState::new().unwrap();
        
        // Check that the initial pattern is set correctly
        let expected_cells = [(5, 5), (5, 6), (5, 7), (6, 6), (6, 7), (6, 8)];
        
        for (x, y) in expected_cells.iter() {
            assert_eq!(state.get_cell(0, *x, *y), 1, "Cell ({}, {}) should be alive", x, y);
        }
        
        // Check a few cells that should be dead
        assert_eq!(state.get_cell(0, 0, 0), 0);
        assert_eq!(state.get_cell(0, 4, 5), 0);
        assert_eq!(state.get_cell(0, 7, 7), 0);
    }

    #[test]
    fn test_is_cell_alive() {
        let mut state = MainState::new().unwrap();
        
        // Test with active buffer
        state.set_cell(state.active_buffer, 15, 15, 1);
        assert!(state.is_cell_alive(15, 15));
        assert_eq!(state.is_cell_alive_i32(15, 15), 1);
        
        // Test dead cell
        assert!(!state.is_cell_alive(16, 15));
        assert_eq!(state.is_cell_alive_i32(16, 15), 0);
        
        // Test out of bounds
        assert!(!state.is_cell_alive(MAP_WIDTH, 0));
        assert_eq!(state.is_cell_alive_i32(0, MAP_HEIGHT), 0);
    }

    #[test]
    fn test_number_of_neighbors() {
        let mut state = MainState::new().unwrap();
        
        // Clear all initial cells
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                state.set_cell(state.active_buffer, x, y, 0);
            }
        }
        
        // Create a simple pattern: 3 cells in a row
        // X X X
        state.set_cell(state.active_buffer, 10, 10, 1);
        state.set_cell(state.active_buffer, 11, 10, 1);
        state.set_cell(state.active_buffer, 12, 10, 1);
        
        // Middle cell should have 2 neighbors
        assert_eq!(state.number_of_neighbors(11, 10), 2);
        
        // Left cell should have 1 neighbor
        assert_eq!(state.number_of_neighbors(10, 10), 1);
        
        // Right cell should have 1 neighbor
        assert_eq!(state.number_of_neighbors(12, 10), 1);
        
        // Cell below middle should have 3 neighbors
        assert_eq!(state.number_of_neighbors(11, 11), 3);
        
        // Cell above middle should have 3 neighbors
        assert_eq!(state.number_of_neighbors(11, 9), 3);
    }

    #[test]
    fn test_get_secondary_buffer_index() {
        let mut state = MainState::new().unwrap();
        
        // When active buffer is 0, secondary should be 1
        state.active_buffer = 0;
        assert_eq!(state.get_secondary_buffer_index(), 1);
        
        // When active buffer is 1, secondary should be 0
        state.active_buffer = 1;
        assert_eq!(state.get_secondary_buffer_index(), 0);
    }

    #[test]
    fn test_buffer_independence() {
        let mut state = MainState::new().unwrap();
        
        // Set different patterns in both buffers
        state.set_cell(0, 10, 10, 1);
        state.set_cell(1, 20, 20, 1);
        
        // Verify they don't interfere with each other
        assert_eq!(state.get_cell(0, 10, 10), 1);
        assert_eq!(state.get_cell(0, 20, 20), 0);
        assert_eq!(state.get_cell(1, 10, 10), 0);
        assert_eq!(state.get_cell(1, 20, 20), 1);
    }

    #[test]
    fn test_grid_size() {
        let state = MainState::new().unwrap();
        
        // Verify the grid has the correct total size
        let expected_size = BUFFER_COUNT * MAP_WIDTH * MAP_HEIGHT;
        assert_eq!(state.buffers.len(), expected_size);
    }
}