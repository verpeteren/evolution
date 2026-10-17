use simdeez::prelude::*;

use std::fmt;

#[derive(PartialEq)]
// FBM is the DSL keyword, kept verbatim; Cell1_3 etc. encode noise-type + dimension
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
pub enum Instruction<S: Simd> {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    FBM,
    Ridge,
    Turbulence,
    Cell1,
    Cell2,
    Sqrt,
    Sin,
    Atan,
    Atan2,
    Tan,
    Log,
    Abs,
    Floor,
    Ceil,
    Clamp,
    Wrap,
    Square,
    Max,
    Min,
    Mandelbrot,
    FBM3,
    FBM4,
    Ridge3,
    Ridge4,
    Turbulence3,
    Turbulence4,
    Cell1_3,
    Cell1_4,
    Cell2_3,
    Cell2_4,
    Picture(String),
    Constant(S::Vf32),
    Width,
    Height,
    PI,
    E,
    X,
    Y,
    T,
}

impl<S> fmt::Debug for Instruction<S>
where
    S: Simd,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let name = match self {
            Instruction::Add => "Add".to_string(),
            Instruction::Sub => "Sub".to_string(),
            Instruction::Mul => "Mul".to_string(),
            Instruction::Div => "Div".to_string(),
            Instruction::Mod => "Mod".to_string(),
            Instruction::FBM => "FBM".to_string(),
            Instruction::Ridge => "Ridge".to_string(),
            Instruction::Turbulence => "Turbulence".to_string(),
            Instruction::Cell1 => "Cell1".to_string(),
            Instruction::Cell2 => "Cell2".to_string(),
            Instruction::Sqrt => "Sqrt".to_string(),
            Instruction::Sin => "Sin".to_string(),
            Instruction::Atan => "Atan".to_string(),
            Instruction::Atan2 => "Atan2".to_string(),
            Instruction::Tan => "Tan".to_string(),
            Instruction::Log => "Log".to_string(),
            Instruction::Abs => "Abs".to_string(),
            Instruction::Floor => "Floor".to_string(),
            Instruction::Ceil => "Ceil".to_string(),
            Instruction::Clamp => "Clamp".to_string(),
            Instruction::Wrap => "Wrap".to_string(),
            Instruction::Square => "Square".to_string(),
            Instruction::Max => "Max".to_string(),
            Instruction::Min => "Min".to_string(),
            Instruction::Mandelbrot => "Mandelbrot".to_string(),
            Instruction::FBM3 => "FBM3".to_string(),
            Instruction::FBM4 => "FBM4".to_string(),
            Instruction::Ridge3 => "Ridge3".to_string(),
            Instruction::Ridge4 => "Ridge4".to_string(),
            Instruction::Turbulence3 => "Turbulence3".to_string(),
            Instruction::Turbulence4 => "Turbulence4".to_string(),
            Instruction::Cell1_3 => "Cell1_3".to_string(),
            Instruction::Cell1_4 => "Cell1_4".to_string(),
            Instruction::Cell2_3 => "Cell2_3".to_string(),
            Instruction::Cell2_4 => "Cell2_4".to_string(),
            Instruction::Picture(pic_name) => format!("Picture({})", pic_name),
            Instruction::Constant(vf32) => format!("Constant({:?}", vf32),
            Instruction::Width => "Width".to_string(),
            Instruction::Height => "Height".to_string(),
            Instruction::PI => "PI".to_string(),
            Instruction::E => "E".to_string(),
            Instruction::X => "X".to_string(),
            Instruction::Y => "Y".to_string(),
            Instruction::T => "T".to_string(),
        };

        write!(f, "{}", name)
    }
}

/* impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.isbn == other.isbn
    }
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use simdeez::scalar::Scalar;

    #[test]
    fn test_debug() {
        assert_eq!(&format!("{:?}", Instruction::Add::<Scalar>), "Add");
        assert_eq!(&format!("{:?}", Instruction::Sub::<Scalar>), "Sub");
        assert_eq!(&format!("{:?}", Instruction::Mul::<Scalar>), "Mul");
        assert_eq!(&format!("{:?}", Instruction::Div::<Scalar>), "Div");
        assert_eq!(&format!("{:?}", Instruction::Mod::<Scalar>), "Mod");
        assert_eq!(&format!("{:?}", Instruction::FBM::<Scalar>), "FBM");
        assert_eq!(&format!("{:?}", Instruction::Ridge::<Scalar>), "Ridge");
        assert_eq!(
            &format!("{:?}", Instruction::Turbulence::<Scalar>),
            "Turbulence"
        );
        assert_eq!(&format!("{:?}", Instruction::Cell1::<Scalar>), "Cell1");
        assert_eq!(&format!("{:?}", Instruction::Cell2::<Scalar>), "Cell2");
        assert_eq!(&format!("{:?}", Instruction::Sqrt::<Scalar>), "Sqrt");
        assert_eq!(&format!("{:?}", Instruction::Sin::<Scalar>), "Sin");
        assert_eq!(&format!("{:?}", Instruction::Atan::<Scalar>), "Atan");
        assert_eq!(&format!("{:?}", Instruction::Atan2::<Scalar>), "Atan2");
        assert_eq!(&format!("{:?}", Instruction::Tan::<Scalar>), "Tan");
        assert_eq!(&format!("{:?}", Instruction::Log::<Scalar>), "Log");
        assert_eq!(&format!("{:?}", Instruction::Abs::<Scalar>), "Abs");
        assert_eq!(&format!("{:?}", Instruction::Floor::<Scalar>), "Floor");
        assert_eq!(&format!("{:?}", Instruction::Ceil::<Scalar>), "Ceil");
        assert_eq!(&format!("{:?}", Instruction::Clamp::<Scalar>), "Clamp");
        assert_eq!(&format!("{:?}", Instruction::Wrap::<Scalar>), "Wrap");
        assert_eq!(&format!("{:?}", Instruction::Square::<Scalar>), "Square");
        assert_eq!(&format!("{:?}", Instruction::Max::<Scalar>), "Max");
        assert_eq!(&format!("{:?}", Instruction::Min::<Scalar>), "Min");
        assert_eq!(
            &format!("{:?}", Instruction::Mandelbrot::<Scalar>),
            "Mandelbrot"
        );
        assert_eq!(
            &format!("{:?}", Instruction::Picture::<Scalar>("cat.png".to_string())),
            "Picture(cat.png)"
        );
        /*
        assert_eq!(
            &format!("{:?}", Instruction::Constant::<Scalar>(0.03)),
            "Constant(0.03)"
        );
        */
        assert_eq!(&format!("{:?}", Instruction::Width::<Scalar>), "Width");
        assert_eq!(&format!("{:?}", Instruction::Height::<Scalar>), "Height");
        assert_eq!(&format!("{:?}", Instruction::PI::<Scalar>), "PI");
        assert_eq!(&format!("{:?}", Instruction::E::<Scalar>), "E");
        assert_eq!(&format!("{:?}", Instruction::X::<Scalar>), "X");
        assert_eq!(&format!("{:?}", Instruction::Y::<Scalar>), "Y");
        assert_eq!(&format!("{:?}", Instruction::T::<Scalar>), "T");
    }
}
