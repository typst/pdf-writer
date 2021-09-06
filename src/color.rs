use super::*;

/// A color space.
///
/// These are either the predefined, parameter-less color spaces like
/// `DeviceGray` or the ones defined by the user, accessed through the `Named`
/// variant. A custom color space of types like `CalRGB` or `Pattern` can be set
/// by registering it with the [`/ColorSpace`](ColorSpaces) dictionary in the
/// current [`Resources`] dictionary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum ColorSpace<'a> {
    DeviceGray,
    DeviceRgb,
    DeviceCmyk,
    Pattern,
    Named(Name<'a>),
}

impl<'a> ColorSpace<'a> {
    pub(crate) fn to_name(self) -> Name<'a> {
        match self {
            Self::DeviceGray => Name(b"DeviceGray"),
            Self::DeviceRgb => Name(b"DeviceRGB"),
            Self::DeviceCmyk => Name(b"DeviceCMYK"),
            Self::Pattern => Name(b"Pattern"),
            Self::Named(name) => name,
        }
    }
}

/// Writer for a _color space dictionary_.
///
/// This struct is created by [`Resources::color_spaces`].
pub struct ColorSpaces<'a> {
    dict: Dict<&'a mut PdfWriter>,
}

impl<'a> ColorSpaces<'a> {
    pub(crate) fn new(obj: Obj<&'a mut PdfWriter>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write a `CalGray` color space.
    pub fn cal_gray(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::CalGray.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.pair(Name(b"Gamma"), gamma);
        }

        dict.finish();
        array.finish();

        self
    }

    /// Write a `CalRgb` color space.
    pub fn cal_rgb(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        gamma: Option<f32>,
        matrix: Option<[f32; 9]>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::CalRgb.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(gamma) = gamma {
            dict.pair(Name(b"Gamma"), gamma);
        }

        if let Some(matrix) = matrix {
            dict.key(Name(b"Matrix")).array().typed().items(matrix);
        }

        dict.finish();
        array.finish();

        self
    }

    /// Write a `Lab` color space.
    pub fn lab(
        &mut self,
        name: Name,
        white_point: [f32; 3],
        black_point: Option<[f32; 3]>,
        range: Option<[f32; 4]>,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Lab.to_name());

        let mut dict = array.obj().dict();
        dict.key(Name(b"WhitePoint")).array().typed().items(white_point);

        if let Some(black_point) = black_point {
            dict.key(Name(b"BlackPoint")).array().typed().items(black_point);
        }

        if let Some(range) = range {
            dict.key(Name(b"Range")).array().typed().items(range);
        }

        dict.finish();
        array.finish();

        self
    }

    /// Write an `Indexed` color space. PDF 1.2+.
    ///
    /// The length of the lookup slice must be the product of the dimensions of
    /// the base color space and (`hival + 1`) and `hival` shall be at most 255.
    pub fn indexed(
        &mut self,
        name: Name,
        base: Name,
        hival: i32,
        lookup: &[u8],
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Indexed.to_name());
        array.item(base);
        array.item(hival);
        array.item(Str(lookup));
        array.finish();
        self
    }

    /// Write a `Separation` color space. PDF 1.2+.
    pub fn separation(
        &mut self,
        name: Name,
        color_name: Name,
        base: Name,
        tint: Ref,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Separation.to_name());
        array.item(color_name);
        array.item(base);
        array.item(tint);
        array.finish();
        self
    }

    /// Write a `DeviceN` color space. PDF 1.3+.
    pub fn device_n(
        &mut self,
        name: Name,
        names: impl IntoIterator<Item = Name<'a>>,
        alternate_space: Name,
        tint: Ref,
    ) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::DeviceN.to_name());
        array.obj().array().typed().items(names);
        array.item(alternate_space);
        array.item(tint);
        array.finish();
        self
    }

    /// Write a `Pattern` color space for uncolored patterns. PDF 1.2+.
    ///
    /// The `base` attribute is the color space in which the pattern color is
    /// specified upon use.
    pub fn pattern(&mut self, name: Name, base: Name) -> &mut Self {
        let mut array = self.dict.key(name).array();
        array.item(ColorSpaceType::Pattern.to_name());
        array.item(base);
        array.finish();
        self
    }
}

deref!('a, ColorSpaces<'a> => Dict<&'a mut PdfWriter>, dict);

/// A color space type that requires further parameters. These are for internal
/// use. Instances of these color spaces may be used by defining them and their
/// parameters through the [`/ColorSpace`](ColorSpaces) dictionary.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ColorSpaceType {
    CalGray,
    CalRgb,
    Lab,
    #[allow(unused)]
    IccBased,
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

impl ColorSpaceType {
    fn to_name(self) -> Name<'static> {
        match self {
            Self::CalGray => Name(b"CalGray"),
            Self::CalRgb => Name(b"CalRGB"),
            Self::Lab => Name(b"Lab"),
            Self::IccBased => Name(b"ICCBased"),
            Self::Indexed => Name(b"Indexed"),
            Self::Pattern => Name(b"Pattern"),
            Self::Separation => Name(b"Separation"),
            Self::DeviceN => Name(b"DeviceN"),
        }
    }
}

/// Writer for a _tiling pattern stream_.
pub struct TilingStream<'a> {
    stream: Stream<'a>,
}

impl<'a> TilingStream<'a> {
    pub(crate) fn start(mut stream: Stream<'a>) -> Self {
        stream.pair(Name(b"Type"), Name(b"Pattern"));
        stream.pair(Name(b"PatternType"), PatternType::Tiling.to_int());
        Self { stream }
    }

    /// Write the `/PaintType` attribute.
    ///
    /// Sets whether to use external or stream color. Required.
    pub fn paint_type(&mut self, paint_type: PaintType) -> &mut Self {
        self.stream.pair(Name(b"PaintType"), paint_type.to_int());
        self
    }

    /// Write the `/TilingType` attribute.
    ///
    /// Sets how to stretch and space the pattern. Required.
    pub fn tiling_type(&mut self, tiling_type: TilingType) -> &mut Self {
        self.stream.pair(Name(b"TilingType"), tiling_type.to_int());
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the pattern in the pattern's coordinate system.
    /// Required.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.stream.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/XStep` attribute.
    ///
    /// Sets the horizontal spacing between pattern cells. Required.
    ///
    /// Panics if `x_step` is zero.
    pub fn x_step(&mut self, x_step: f32) -> &mut Self {
        assert!(x_step != 0.0, "x step must not be zero");
        self.stream.pair(Name(b"XStep"), x_step);
        self
    }

    /// Write the `/YStep` attribute.
    ///
    /// Sets the vertical spacing between pattern cells. Required.
    ///
    /// Panics if `y_step` is zero.
    pub fn y_step(&mut self, y_step: f32) -> &mut Self {
        assert!(y_step != 0.0, "y step must not be zero");
        self.stream.pair(Name(b"YStep"), y_step);
        self
    }

    /// Start writing the `/Resources` dictionary.
    ///
    /// Sets the resources used by the pattern. Required.
    pub fn resources(&mut self) -> Resources<'_> {
        Resources::new(self.key(Name(b"Resources")))
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the pattern coordinate system to the parent content stream
    /// coordinates. The default is the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.stream.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }
}

deref!('a, TilingStream<'a> => Stream<'a>, stream);

/// Type of paint for a tiling pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PaintType {
    /// Paint the pattern with the colors specified in the stream.
    Colored,
    /// Paint the pattern with the colors active when the pattern was painted.
    Uncolored,
}

impl PaintType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Colored => 1,
            Self::Uncolored => 2,
        }
    }
}

/// How to adjust tile spacing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TilingType {
    /// Constant space between each tile, tiles may be distorted by 1px.
    ConstantSpacing,
    /// Tile size is constant, spacing between may vary by 1px.
    NoDistortion,
    /// Constant space between each tile and faster drawing, tiles may be distorted.
    FastConstantSpacing,
}

impl TilingType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::ConstantSpacing => 1,
            Self::NoDistortion => 2,
            Self::FastConstantSpacing => 3,
        }
    }
}

/// Writer for a _shading pattern_.
pub struct ShadingPattern<'a> {
    dict: Dict<IndirectGuard<'a>>,
}

impl<'a> ShadingPattern<'a> {
    pub(crate) fn start(obj: Obj<IndirectGuard<'a>>) -> Self {
        let mut dict = obj.dict();
        dict.pair(Name(b"Type"), Name(b"Pattern"));
        dict.pair(Name(b"PatternType"), PatternType::Shading.to_int());
        Self { dict }
    }

    /// Start writing the `/Shading` dictionary.
    pub fn shading(&mut self) -> Shading<'_> {
        Shading::start(self.dict.key(Name(b"Shading")))
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Sets the matrix to use for the pattern. Defaults to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }
}

deref!('a, ShadingPattern<'a> => Dict< IndirectGuard<'a>>, dict);

/// Writer for a _shading dictionary_.
///
/// This struct is created by [`ShadingPattern::shading`].
pub struct Shading<'a> {
    dict: Dict<&'a mut PdfWriter>,
}

impl<'a> Shading<'a> {
    pub(crate) fn start(obj: Obj<&'a mut PdfWriter>) -> Self {
        Self { dict: obj.dict() }
    }

    /// Write the `/ShadingType` attribute.
    ///
    /// Sets the type of shading. The available and required attributes change
    /// depending on this. Required.
    pub fn shading_type(&mut self, shading_type: ShadingType) -> &mut Self {
        self.dict.pair(Name(b"ShadingType"), shading_type.to_int());
        self
    }

    /// Write the `/ColorSpace` attribute.
    ///
    /// Sets the color space of the shading function. May not be a `Pattern`
    /// space. Required.
    pub fn color_space(&mut self, color_space: ColorSpace) -> &mut Self {
        self.dict.pair(Name(b"ColorSpace"), color_space.to_name());
        self
    }

    /// Write the `/Background` attribute.
    ///
    /// Sets the background color of the area to be shaded. The `background`
    /// iterator must contain exactly as many elements as the current
    /// `ColorSpace` has dimensions.
    pub fn background(&mut self, background: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.key(Name(b"Background")).array().typed().items(background);
        self
    }

    /// Write the `/BBox` attribute.
    ///
    /// Sets the bounding box of the shading in the target coordinate system.
    pub fn bbox(&mut self, bbox: Rect) -> &mut Self {
        self.dict.pair(Name(b"BBox"), bbox);
        self
    }

    /// Write the `/AntiAlias` attribute.
    ///
    /// Sets whether to anti-alias the shading.
    pub fn anti_alias(&mut self, anti_alias: bool) -> &mut Self {
        self.dict.pair(Name(b"AntiAlias"), anti_alias);
        self
    }

    /// Write the `/Domain` attribute.
    ///
    /// Sets the domain of the shading function in a rectangle. Can be used for
    /// function, axial, or radial shadings. Will otherwise default to
    /// `[x_min = 0, x_max = 1, y_min = 0, y_max = 1]`
    pub fn domain(&mut self, domain: [f32; 4]) -> &mut Self {
        self.dict.key(Name(b"Domain")).array().typed().items(domain);
        self
    }

    /// Write the `/Matrix` attribute.
    ///
    /// Maps the shading domain rectangle to the target coordinate system. Can
    /// be used for function shadings. Will otherwise
    /// default to the identity matrix.
    pub fn matrix(&mut self, matrix: [f32; 6]) -> &mut Self {
        self.dict.key(Name(b"Matrix")).array().typed().items(matrix);
        self
    }

    /// Write the `/Function` attribute.
    ///
    /// Sets the 2-in function to use for shading. Required.
    pub fn function(&mut self, function: Ref) -> &mut Self {
        self.dict.pair(Name(b"Function"), function);
        self
    }

    /// Write the `/Coords` attribute.
    ///
    /// Sets the coordinates of the start and end of the axis in terms of the
    /// target coordinate system. Required for axial (4 items) and radial (6
    /// items; centers and radii) shadings.
    pub fn coords(&mut self, coords: impl IntoIterator<Item = f32>) -> &mut Self {
        self.dict.key(Name(b"Coords")).array().typed().items(coords);
        self
    }

    /// Write the `/Extend` attribute.
    ///
    /// Set whether the shading should extend beyond either side of the axis /
    /// circles. Can be used for axial and radial shadings.
    pub fn extend(&mut self, extend: [bool; 2]) -> &mut Self {
        self.dict.key(Name(b"Extend")).array().typed().items(extend);
        self
    }
}

deref!('a, Shading<'a> => Dict<&'a mut PdfWriter>, dict);

/// What kind of shading to use.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ShadingType {
    /// The function specifies the color for each point in the domain.
    Function,
    /// The function specifies the color for each point on a line.
    Axial,
    /// The function specifies the color for each circle between two nested circles.
    Radial,
}

impl ShadingType {
    pub(crate) fn to_int(self) -> i32 {
        match self {
            Self::Function => 1,
            Self::Axial => 2,
            Self::Radial => 3,
        }
    }
}

/// Type of pattern.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum PatternType {
    /// A tiling pattern.
    Tiling,
    /// A shading pattern.
    Shading,
}

impl PatternType {
    fn to_int(self) -> i32 {
        match self {
            Self::Tiling => 1,
            Self::Shading => 2,
        }
    }
}
