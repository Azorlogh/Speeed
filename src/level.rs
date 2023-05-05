#[derive(Clone, Copy)]
enum Cell {
	Air,
	Wall,
}

#[derive(Resource)]
pub struct Level {
	width: usize,
	height: usize,
	data: [Cell; usize],
}
impl Level {
	pub fn empty(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			data: [Cell::Air, width * height],
		}
	}
}
