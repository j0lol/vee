use crate::{error::CharConversionError, nx::UuidVer4, seal::Sealant};

pub struct Position {
    pub x: u8,
    pub y: u8,
}

pub struct PositionY {
    pub y: u8,
}

/// 'Scale' and 'Aspect'.
pub struct Scale {
    pub w: u8,
    pub h: u8,
}

/// 'Scale'.
pub struct ScaleX {
    pub w: u8,
}

/// 'Aspect'.
pub struct ScaleY {
    pub h: u8,
}

/// Scale and aspect, at the same time.
pub struct UniformScale {
    pub amount: u8,
}

/// The actual angle is somewhat dependant on what the shape is.
/// Think of this as more of a... rotation difference.
pub struct Rotation {
    pub ang: u8,
}

/// Represents a color from any Mii format with type-safe table references.
///
/// Cafe and RVL/NTR use separate color tables for each face part,
/// while Nx uses a 100-color CommonColor table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericColor {
    /// 6 colors
    CafeFaceline(u8),
    /// 8 colors
    CafeHair(u8),
    /// 6 colors
    CafeEye(u8),
    /// 5 colors
    CafeMouth(u8),
    /// 6 colors
    CafeGlass(u8),

    /// 10 colors
    NxFaceline(u8),
    /// 100 colors
    /// This is used for hair, eye, mouth, glass (and more elsewhere) in Nx
    NxCommon(u8),

    /// 12 colors
    FavoriteColor(u8),
}

impl GenericColor {
    pub fn cafe_faceline(index: u8) -> Self {
        GenericColor::CafeFaceline(index.min(5))
    }

    pub fn cafe_hair(index: u8) -> Self {
        GenericColor::CafeHair(index.min(7))
    }

    pub fn cafe_eye(index: u8) -> Self {
        GenericColor::CafeEye(index.min(5))
    }

    pub fn cafe_mouth(index: u8) -> Self {
        GenericColor::CafeMouth(index.min(4))
    }

    pub fn cafe_glass(index: u8) -> Self {
        GenericColor::CafeGlass(index.min(5))
    }

    pub fn nx_faceline(index: u8) -> Self {
        GenericColor::NxFaceline(index.min(9))
    }

    pub fn nx_common(index: u8) -> Self {
        GenericColor::NxCommon(index.min(99))
    }

    pub fn favorite_color(index: u8) -> Self {
        GenericColor::FavoriteColor(index.min(11))
    }

    pub fn is_nx_common_color(&self) -> bool {
        matches!(self, GenericColor::NxCommon(_))
    }

    pub fn raw_index(&self) -> u8 {
        match self {
            GenericColor::CafeFaceline(i) => *i,
            GenericColor::CafeHair(i) => *i,
            GenericColor::CafeEye(i) => *i,
            GenericColor::CafeMouth(i) => *i,
            GenericColor::CafeGlass(i) => *i,
            GenericColor::NxFaceline(i) => *i,
            GenericColor::NxCommon(i) => *i,
            GenericColor::FavoriteColor(i) => *i,
        }
    }
}

const CAFE_HAIR_TO_NX: [u8; 8] = [8, 1, 2, 3, 4, 5, 6, 7];
const CAFE_EYE_TO_NX: [u8; 6] = [8, 9, 10, 11, 12, 13];
const CAFE_MOUTH_TO_NX: [u8; 5] = [19, 20, 21, 22, 23];
const CAFE_GLASS_TO_NX: [u8; 6] = [8, 14, 15, 16, 17, 18];

impl GenericColor {
    /// Convert this color to a Nx color representation
    ///
    /// This is infallible because all Cafe colors have equivalent Nx colors.
    /// The reverse (Nx -> Cafe) is fallible because Nx has more colors.
    pub fn to_nx(&self) -> GenericColor {
        dbg!(&self);
        match self {
            // First 6 Nx indices match Cafe directly
            GenericColor::CafeFaceline(i) => GenericColor::NxFaceline(*i),

            GenericColor::CafeHair(i) => {
                dbg!(i);
                let nx_index = CAFE_HAIR_TO_NX[*i as usize];
                GenericColor::NxCommon(nx_index)
            }
            GenericColor::CafeEye(i) => {
                let nx_index = CAFE_EYE_TO_NX[*i as usize];
                GenericColor::NxCommon(nx_index)
            }
            GenericColor::CafeMouth(i) => {
                let nx_index = CAFE_MOUTH_TO_NX[*i as usize];
                GenericColor::NxCommon(nx_index)
            }
            GenericColor::CafeGlass(i) => {
                let nx_index = CAFE_GLASS_TO_NX[*i as usize];
                GenericColor::NxCommon(nx_index)
            }

            // Already Nx, return as-is
            GenericColor::NxFaceline(_) | GenericColor::NxCommon(_) => *self,

            // Favorite color is always the same indices on every platform
            GenericColor::FavoriteColor(i) => GenericColor::FavoriteColor(*i),
        }
    }

    /// Convert this color to a Cafe color representation
    ///
    /// This is fallible because Nx has more colors than Cafe.
    /// Returns None if the Nx color doesn't have a Cafe equivalent.
    pub fn to_cafe_faceline(&self) -> Option<GenericColor> {
        match self {
            GenericColor::CafeFaceline(_) => Some(*self),
            GenericColor::NxFaceline(i) if *i < 6 => Some(GenericColor::CafeFaceline(*i)),
            _ => None,
        }
    }

    pub fn to_cafe_hair(&self) -> Option<GenericColor> {
        match self {
            GenericColor::CafeHair(_) => Some(*self),
            GenericColor::NxCommon(nx_idx) => {
                let common_idx = nx_idx;
                CAFE_HAIR_TO_NX
                    .iter()
                    .position(|x| x == common_idx)
                    .map(|pos| GenericColor::CafeHair(pos as u8))
            }
            _ => None,
        }
    }

    pub fn to_cafe_eye(&self) -> Option<GenericColor> {
        match self {
            GenericColor::CafeEye(_) => Some(*self),
            GenericColor::NxCommon(nx_idx) => {
                let common_idx = nx_idx;
                CAFE_EYE_TO_NX
                    .iter()
                    .position(|x| x == common_idx)
                    .map(|pos| GenericColor::CafeEye(pos as u8))
            }
            _ => None,
        }
    }

    pub fn to_cafe_mouth(&self) -> Option<GenericColor> {
        match self {
            GenericColor::CafeMouth(_) => Some(*self),
            GenericColor::NxCommon(nx_idx) => {
                let common_idx = nx_idx;
                CAFE_MOUTH_TO_NX
                    .iter()
                    .position(|x| x == common_idx)
                    .map(|pos| GenericColor::CafeMouth(pos as u8))
            }
            _ => None,
        }
    }

    pub fn to_cafe_glass(&self) -> Option<GenericColor> {
        match self {
            GenericColor::CafeGlass(_) => Some(*self),
            GenericColor::NxCommon(nx_idx) => {
                let common_idx = nx_idx;
                CAFE_GLASS_TO_NX
                    .iter()
                    .position(|x| x == common_idx)
                    .map(|pos| GenericColor::CafeGlass(pos as u8))
            }
            _ => None,
        }
    }
}

// ============================================================================
// Direct conversions for common patterns
// ============================================================================

impl GenericColor {
    /// Convert a Nx color to the appropriate Cafe color type, with a fallback
    ///
    /// If the Nx color can't be represented in Cafe, returns a sensible default
    /// (typically the first color in that table, like black for hair)
    pub fn to_cafe_hair_or_default(&self) -> GenericColor {
        self.to_cafe_hair().unwrap_or(GenericColor::CafeHair(0))
    }

    pub fn to_cafe_eye_or_default(&self) -> GenericColor {
        self.to_cafe_eye().unwrap_or(GenericColor::CafeEye(0))
    }

    pub fn to_cafe_mouth_or_default(&self) -> GenericColor {
        self.to_cafe_mouth().unwrap_or(GenericColor::CafeMouth(0))
    }

    pub fn to_cafe_glass_or_default(&self) -> GenericColor {
        self.to_cafe_glass().unwrap_or(GenericColor::CafeGlass(0))
    }

    pub fn to_cafe_faceline_or_default(&self) -> GenericColor {
        self.to_cafe_faceline()
            .unwrap_or(GenericColor::CafeFaceline(0))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cafe_to_nx_hair() {
        let cafe_black = GenericColor::cafe_hair(0);
        let nx_black = cafe_black.to_nx();

        match nx_black {
            GenericColor::NxCommon(idx) => {
                assert_eq!(idx, 8); // Black is CommonColor[8]
            }
            _ => panic!("Expected NxCommon"),
        }
    }

    #[test]
    fn test_cafe_to_nx_faceline() {
        // First 6 faceline colors map directly
        for i in 0..6 {
            let cafe = GenericColor::cafe_faceline(i);
            let nx = cafe.to_nx();

            match nx {
                GenericColor::NxFaceline(idx) => assert_eq!(idx, i),
                _ => panic!("Expected NxFaceline"),
            }
        }
    }

    #[test]
    fn test_nx_to_cafe_roundtrip() {
        // Convert Cafe -> Nx -> Cafe should preserve the color
        for i in 0..8 {
            let cafe_hair = GenericColor::cafe_hair(i);
            let nx_hair = cafe_hair.to_nx();
            let back = nx_hair.to_cafe_hair().unwrap();
            assert_eq!(cafe_hair, back);
        }
    }

    #[test]
    fn test_nx_to_cafe_unmappable() {
        // Nx color that doesn't exist in Cafe should return None
        let nx_color = GenericColor::nx_common(50);
        assert!(nx_color.to_cafe_hair().is_none());

        // But with default should give us black
        let default = nx_color.to_cafe_hair_or_default();
        assert_eq!(default, GenericColor::cafe_hair(0));
    }

    #[test]
    fn test_raw_index() {
        assert_eq!(GenericColor::cafe_hair(3).raw_index(), 3);
        assert_eq!(GenericColor::nx_common(8).raw_index(), 8);
    }
}

pub struct Eye {
    pub ty: u8,
    pub color: GenericColor,
    pub pos: Position,
    pub scale: Scale,
    pub rotation: Rotation,
}
pub struct Eyebrow {
    pub ty: u8,
    pub color: GenericColor,
    pub pos: Position,
    pub scale: Scale,
    pub rotation: Rotation,
}
pub struct Nose {
    pub ty: u8,
    pub pos: PositionY,
    pub scale: UniformScale,
}
pub struct Mouth {
    pub ty: u8,
    pub color: GenericColor,
    pub pos: PositionY,
    pub scale: Scale,
}

/// The face model itself.
pub struct Faceline {
    pub ty: u8,
    pub color: GenericColor,
    pub wrinkle_ty: u8,
    pub makeup_ty: u8,
}

pub struct Hair {
    pub ty: u8,
    pub color: GenericColor,
    pub flip: bool,
}

pub struct Beard {
    pub ty: u8,
    pub color: GenericColor,
}

pub struct Mustache {
    pub ty: u8,
    pub pos: PositionY,
    pub scale: ScaleX,
}

pub struct Glass {
    pub ty: u8,
    pub color: GenericColor,
    pub pos: PositionY,
    pub scale: ScaleX,
}

pub struct Mole {
    pub ty: u8,
    pub pos: Position,
    pub scale: ScaleX,
}

#[derive(PartialEq, Debug)]
pub enum Gender {
    Male,
    Female,
}
impl Gender {
    pub fn from_bool(b: bool) -> Gender {
        if b { Gender::Female } else { Gender::Male }
    }
    pub fn from_u8(u: u8) -> Result<Gender, CharConversionError> {
        match u {
            0_u8 => Ok(Gender::Male),
            1_u8 => Ok(Gender::Female),
            _ => Err(CharConversionError::FieldOob("gender".to_string())),
        }
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Gender::Male => 0,
            Gender::Female => 1,
        }
    }
}

/// The body shape of the Char.
pub struct Body {
    pub gender: Gender, // Now referred to as "style".
    pub height: u8,
    pub build: u8,
}

pub struct MetaData {
    pub special: bool,
    pub favorite_color: GenericColor,
}

pub struct RvlCreationData {/* todo */}
pub struct CtrCreationData {/* todo */}
pub struct NxCreationData {
    pub create_info: UuidVer4,
    pub font_region: u8,
    pub region_move: u8,
}

pub enum CreationData {
    None,
    Rvl(RvlCreationData),
    Ctr(CtrCreationData),
    Nx(NxCreationData),
}

/// Generic `Char` information.
/// Names here are based on names in target-specific structs, but not representative.
pub struct GenericChar {
    /// Sometimes called "nickname"
    pub name: String,

    pub meta_data: MetaData,
    pub creation_data: CreationData,
    pub body: Body,
    pub faceline: Faceline,
    pub hair: Hair,
    pub eye: Eye,
    pub eyebrow: Eyebrow,
    pub nose: Nose,
    pub mouth: Mouth,
    pub beard: Beard,
    pub mustache: Mustache,
    pub glass: Glass,
    pub mole: Mole,
}

/// This trait is sealed (you can't implement this on your own items.)
pub trait AsGenericChar: Sealant {
    /// Convert this representation into [GenericChar]
    fn as_generic(&self) -> Result<GenericChar, CharConversionError>;
}

/// This trait is sealed (you can't implement this on your own items.)
pub trait FromGenericChar: Sealant {
    /// Hopefully the type you are implementing on.
    type Output;

    /// Make this char representation from [GenericChar]
    fn from_generic(char: GenericChar) -> Self::Output;
}
