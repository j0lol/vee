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

/// The actual angle is somewhat dependant on what the shape is.
/// Think of this as more of a... rotation difference.
pub struct Rotation {
    pub ang: u8,
}

// Maybe make this *more* specific? Like point to the correct color table.
pub enum GenericColor {
    CafeTable(usize),
    NxTable(usize),
}

/// Just so happens to be the same on every platform. Cool!
pub struct FavoriteColor(usize);

pub struct Eye {
    pub ty: u8,
    pub color: GenericColor,
    pub pos: Position,
    pub scale: ScaleY,
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
    pub scale: ScaleX,
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
    pub color: GenericColor,
    pub scale: ScaleY,
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

pub enum Gender {
    Male,
    Female,
}

/// The body shape of the Char.
pub struct Body {
    pub gender: Gender, // Now referred to as "style".
    pub height: u8,
    pub build: u8,
}

pub struct MetaData {
    pub special: bool,
    pub favorite_color: FavoriteColor,
}

pub struct CtrCreationData {/* todo */}
pub struct NxCreationData {/* todo */}

pub enum CreationData {
    Ctr(CtrCreationData),
    Nx(NxCreationData),
}

/// Generic `Char` information.
/// Names here are based on names in target-specific structs, but not representative.
pub struct GenericChar {
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
