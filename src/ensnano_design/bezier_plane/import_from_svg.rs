use crate::ensnano_design::bezier_plane::{BezierPath, BezierPlaneId, BezierVertex};
use svg::{
    node::element::path::{Command, Data, Position},
    parser::Event,
};
use ultraviolet::{Vec2, Vec3};

const SCALE: Vec2 = Vec2 { x: 0.1, y: 0.1 };
const ORIGIN: Vec2 = Vec2 {
    x: 134.23425,
    y: 13.5557,
};

pub fn read_first_svg_path(file_path: &std::path::Path) -> Result<BezierPath, SvgImportError> {
    let mut content = String::new();
    let events = svg::open(file_path, &mut content)?;

    for event in events {
        if let Event::Tag(_, _, attributes) = event {
            let data = attributes
                .get("d")
                .ok_or_else(|| SvgImportError::AttributeNotFound(String::from("d")))?;
            let data = Data::parse(data)?;

            let mut ret = PathBuilder::default();
            for command in data.iter() {
                match command {
                    Command::Close => return Ok(ret.close()),
                    Command::Move(Position::Absolute, parameters) => match parameters.as_ref() {
                        [x, y] => {
                            let at = Vec2::new(*x, *y);
                            ret.start(at)?;
                        }
                        _ => return Err(SvgImportError::BadParameters),
                    },
                    Command::CubicCurve(Position::Absolute, parameters) => {
                        let arg = MoveToParameter::from_svg_parameter(parameters)?;
                        ret.move_to(arg)?;
                    }
                    _ => (),
                }
            }
            return Ok(ret.finish());
        }
    }
    Err(SvgImportError::NoPathFound)
}

#[derive(Default)]
struct PathBuilder {
    vertices: Vec<BezierVertex>,
}

impl PathBuilder {
    fn start(&mut self, at: Vec2) -> Result<(), SvgImportError> {
        if self.vertices.is_empty() {
            self.vertices = vec![BezierVertex {
                plane_id: BezierPlaneId(0),
                position: SCALE * at - ORIGIN,
                position_in: None,
                position_out: None,
                grid_translation: Vec3::zero(),
                angle_with_plane: 0.,
            }];
        } else {
            return Err(SvgImportError::UnexpectedCommand(String::from("Move")));
        }

        Ok(())
    }

    fn move_to(&mut self, parameters: MoveToParameter) -> Result<(), SvgImportError> {
        let prev_vertex = self
            .vertices
            .last_mut()
            .ok_or_else(|| SvgImportError::UnexpectedCommand(String::from("CubicCurve")))?;
        prev_vertex.position_out = Some(SCALE * parameters.control_1 - ORIGIN);

        let new_vertex = BezierVertex {
            plane_id: BezierPlaneId(0),
            position: SCALE * parameters.position - ORIGIN,
            position_out: None,
            position_in: Some(SCALE * parameters.control_2 - ORIGIN),
            grid_translation: Vec3::zero(),
            angle_with_plane: 0.,
        };
        self.vertices.push(new_vertex);

        Ok(())
    }

    fn close(self) -> BezierPath {
        BezierPath {
            vertices: self.vertices,
            is_cyclic: true,
            grid_type: None,
        }
    }

    fn finish(self) -> BezierPath {
        BezierPath {
            vertices: self.vertices,
            is_cyclic: false,
            grid_type: None,
        }
    }
}

struct MoveToParameter {
    position: Vec2,
    control_1: Vec2,
    control_2: Vec2,
}

impl MoveToParameter {
    fn from_svg_parameter(parameters: &[f32]) -> Result<Self, SvgImportError> {
        match parameters {
            [c1x, c1y, c2x, c2y, px, py] => Ok(Self {
                control_1: Vec2::new(*c1x, *c1y),
                control_2: Vec2::new(*c2x, *c2y),
                position: Vec2::new(*px, *py),
            }),
            _ => Err(SvgImportError::BadParameters),
        }
    }
}

#[derive(Debug)]
pub enum SvgImportError {
    IOError(#[expect(unused)] std::io::Error),
    SvgParserError(#[expect(unused)] svg::parser::Error),
    NoPathFound,
    AttributeNotFound(#[expect(unused)] String),
    UnexpectedCommand(#[expect(unused)] String),
    BadParameters,
}

impl From<std::io::Error> for SvgImportError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<svg::parser::Error> for SvgImportError {
    fn from(e: svg::parser::Error) -> Self {
        Self::SvgParserError(e)
    }
}
