// NOTE: this may be the worst piece of code ever written

use iced::{
    advanced::{
        graphics::Compositor as Comp, image::Renderer as ImageRend, svg::Renderer as SvgRend,
        text::Renderer as TextRend, Renderer as NormRend,
    },
    futures::FutureExt,
};

pub struct Renderer<Rend>(Rend);

impl<Rend: NormRend> NormRend for Renderer<Rend> {
    fn start_layer(&mut self, bounds: iced::Rectangle) {
        self.0.start_layer(bounds)
    }

    fn end_layer(&mut self) {
        self.0.end_layer()
    }

    fn start_transformation(&mut self, transformation: iced::Transformation) {
        self.0.start_transformation(transformation)
    }

    fn end_transformation(&mut self) {
        self.0.end_transformation();
    }

    fn fill_quad(
        &mut self,
        quad: iced::advanced::renderer::Quad,
        background: impl Into<iced::Background>,
    ) {
        self.0.fill_quad(quad, background);
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

impl<Rend: TextRend<Font = iced::Font>> TextRend for Renderer<Rend> {
    type Font = iced::Font;
    type Paragraph = Rend::Paragraph;
    type Editor = crate::editor::Editor;

    const ICON_FONT: Self::Font = Rend::ICON_FONT;
    const CHECKMARK_ICON: char = Rend::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = Rend::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        self.0.default_font()
    }

    fn default_size(&self) -> iced::Pixels {
        self.0.default_size()
    }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: iced::Point,
        color: iced::Color,
        clip_bounds: iced::Rectangle,
    ) {
        self.0.fill_paragraph(text, position, color, clip_bounds);
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: iced::Point,
        _color: iced::Color,
        _clip_bounds: iced::Rectangle,
    ) {
        // self.fill_editor(editor, position, color, clip_bounds);
    }

    fn fill_text(
        &mut self,
        text: iced::advanced::Text<String, Self::Font>,
        position: iced::Point,
        color: iced::Color,
        clip_bounds: iced::Rectangle,
    ) {
        self.0.fill_text(text, position, color, clip_bounds);
    }
}

impl<Rend: ImageRend> ImageRend for Renderer<Rend> {
    type Handle = Rend::Handle;

    fn measure_image(&self, handle: &Self::Handle) -> iced::Size<u32> {
        self.0.measure_image(handle)
    }

    fn draw_image(
        &mut self,
        image: iced::advanced::image::Image<Self::Handle>,
        bounds: iced::Rectangle,
    ) {
        self.0.draw_image(image, bounds);
    }
}

impl<Rend: SvgRend> SvgRend for Renderer<Rend> {
    fn measure_svg(&self, handle: &iced::advanced::svg::Handle) -> iced::Size<u32> {
        self.0.measure_svg(handle)
    }

    fn draw_svg(&mut self, svg: iced::advanced::svg::Svg, bounds: iced::Rectangle) {
        self.0.draw_svg(svg, bounds);
    }
}

pub struct Compositor<Comp>(Comp);

impl<C: Comp<Renderer = R, Surface = S>, R: NormRend, S> Comp for Compositor<C> {
    type Renderer = Renderer<R>;
    type Surface = S;

    fn with_backend<W: iced::advanced::graphics::compositor::Window + Clone>(
        settings: iced::advanced::graphics::Settings,
        compatible_window: W,
        backend: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Self, iced::advanced::graphics::Error>> {
        C::with_backend(settings, compatible_window, backend).map(|r| r.map(Compositor))
    }

    fn create_renderer(&self) -> Self::Renderer {
        Renderer(self.0.create_renderer())
    }

    fn create_surface<W: iced::advanced::graphics::compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        self.0.create_surface(window, width, height)
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        self.0.configure_surface(surface, width, height);
    }

    fn fetch_information(&self) -> iced::advanced::graphics::compositor::Information {
        self.0.fetch_information()
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced::widget::shader::Viewport,
        background_color: iced::Color,
        overlay: &[T],
    ) -> Result<(), iced::advanced::graphics::compositor::SurfaceError> {
        self.0.present(
            &mut renderer.0,
            surface,
            viewport,
            background_color,
            overlay,
        )
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced::widget::shader::Viewport,
        background_color: iced::Color,
        overlay: &[T],
    ) -> Vec<u8> {
        self.0.screenshot(
            &mut renderer.0,
            surface,
            viewport,
            background_color,
            overlay,
        )
    }
}

// NOTE: all that generic code above gets thrown out because of this

impl iced::advanced::graphics::compositor::Default for Renderer<iced::Renderer> {
    type Compositor = Compositor<iced_renderer::Compositor>;
}
