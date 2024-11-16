//! A custom [`GpuDriver`] implementation for the `glium` backend.

use std::{borrow::Cow, collections::HashMap, rc::Rc, sync::mpsc};

use glium::{
    backend::{Context, Facade},
    framebuffer::SimpleFrameBuffer,
    program,
    texture::{ClientFormat, MipmapsOption, RawImage2d, SrgbTexture2d, UncompressedFloatFormat},
    uniform,
    uniforms::UniformBuffer,
    vertex::{AttributeType, VertexBufferAny},
    Blend, DrawParameters, Program, Surface, Texture2d,
};

use crate::{
    bitmap::{BitmapFormat, OwnedBitmap},
    gpu_driver::ShaderType,
};

use super::{GpuCommand, GpuDriver, IndexBuffer, RenderBuffer, VertexBuffer, VertexBufferFormat};
pub use either_texture::{EitherSampler, EitherTexture};

mod either_texture;

type StaticVertexFormatBinding =
    Cow<'static, [(Cow<'static, str>, usize, i32, AttributeType, bool)]>;

lazy_static::lazy_static! {
    static ref BINDING_2F4UB2F2F28F: StaticVertexFormatBinding = Cow::Owned(vec![
        (
            Cow::Borrowed("in_Position"),
            0,
            -1,
            glium::vertex::AttributeType::F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Color"),
            2 * ::std::mem::size_of::<f32>(),
            -1,
            glium::vertex::AttributeType::U8U8U8U8,
            true,
        ),
        (
            Cow::Borrowed("in_TexCoord"),
            2 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_ObjCoord"),
            4 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data0"),
            6 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data1"),
            10 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data2"),
            14 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data3"),
            18 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data4"),
            22 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data5"),
            26 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Data6"),
            30 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32F32F32,
            false,
        ),
    ]);

    static ref BINDING_2F4UB2F: StaticVertexFormatBinding = Cow::Owned(vec![
        (
            Cow::Borrowed("in_Position"),
            0,
            -1,
            glium::vertex::AttributeType::F32F32,
            false,
        ),
        (
            Cow::Borrowed("in_Color"),
            2 * ::std::mem::size_of::<f32>(),
            -1,
            glium::vertex::AttributeType::U8U8U8U8,
            true,
        ),
        (
            Cow::Borrowed("in_TexCoord"),
            2 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
            -1,
            glium::vertex::AttributeType::F32F32,
            false,
        ),
    ]);

}

/// Errors can occure when calling [`create_gpu_driver`]
#[derive(Debug, thiserror::Error)]
pub enum GliumGpuDriverError {
    #[error("Failed to create `glium` textures")]
    TextureCreationError(#[from] glium::texture::TextureCreationError),
    #[error("Failed to shader program in `glium`")]
    ProgramCreationError(#[from] glium::program::ProgramChooserCreationError),
    #[error("Failed to create `glium` index buffer")]
    IndexBufferCreationError(#[from] glium::index::BufferCreationError),
    #[error("Failed to create `glium` vertex buffer")]
    VertexBufferCreationError(#[from] glium::vertex::BufferCreationError),
    #[error("Failed to create `glium` buffer")]
    BufferCreationError(#[from] glium::buffer::BufferCreationError),
    #[error("Failed to create `glium` framebuffer")]
    FrameBufferCreationError(#[from] glium::framebuffer::ValidationError),
    #[error("Failed to draw")]
    DrawError(#[from] glium::DrawError),
    #[error(
        "The index offset ({draw_index_offset}) and size ({draw_index_size}) used in draw is out of range from the selected index buffer (size = {index_buffer_size})"
    )]
    DrawIndexOutOfRange {
        index_buffer_size: usize,
        draw_index_offset: u32,
        draw_index_size: u32,
    },
}

/// Creates a GPU driver for `glium`.
///
/// `glium` context must run in one thread, but the `gpu_driver` require `Send`,
/// since it works by creating multiple callbacks to `ultralight` C library.
///
/// We cannot guarantee that `ultralight` callbacks will be called in the same thread as `glium`,
/// and thus, we create two objects. One is the sender, which implements `GpuDriver`,
/// and should be used by the `ultralight` library. And the other is the receiver,
/// which handles all gpu rendering logic.
///
/// **Make sure that both the sender and the receiver are alive for the whole**
/// **lifetime of the [`Renderer`](crate::renderer::Renderer)**
///
/// # Examples
/// ```no_run,ignore
/// let (sender, mut receiver) = create_gpu_driver(&display);
/// platform::set_gpu_driver(lib.clone(), sender);
///
/// renderer.render(); // will dispatch and send all events to `reciever` from `ultralight`
/// receiver.render(); // will render all events received from `sender`
/// ```
pub fn create_gpu_driver<F>(
    facade: &F,
) -> Result<(GliumGpuDriverSender, GliumGpuDriverReceiver), GliumGpuDriverError>
where
    F: Facade + ?Sized,
{
    let (sender, receiver) = mpsc::channel();
    Ok((
        GliumGpuDriverSender {
            next_texture_id: 0,
            next_render_buffer_id: 0,
            next_geometry_id: 0,
            sender,
        },
        GliumGpuDriverReceiver::new(receiver, facade.get_context())?,
    ))
}

enum GliumGpuCommand {
    CreateTexture(u32, OwnedBitmap),
    UpdateTexture(u32, OwnedBitmap),
    DestroyTexture(u32),
    CreateRenderBuffer(u32, RenderBuffer),
    DestroyRenderBuffer(u32),
    CreateGeometry(u32, GliumDriverVertexBuffer, IndexBuffer),
    UpdateGeometry(u32, GliumDriverVertexBuffer, IndexBuffer),
    DestroyGeometry(u32),
    UpdateCommandList(Vec<GpuCommand>),
}

// a wrapper around the vertex buffer format
enum GliumDriverVertexBuffer {
    Format2f4ub2f(Vec<ul_sys::ULVertex_2f_4ub_2f>),
    Format2f4ub2f2f28f(Vec<ul_sys::ULVertex_2f_4ub_2f_2f_28f>),
}

impl GliumDriverVertexBuffer {
    fn into_glium_vertex_buffer<F>(
        self,
        context: &F,
    ) -> Result<VertexBufferAny, GliumGpuDriverError>
    where
        F: Facade + ?Sized,
    {
        match self {
            GliumDriverVertexBuffer::Format2f4ub2f(buf) => {
                let element_size = std::mem::size_of::<ul_sys::ULVertex_2f_4ub_2f>();

                // SAFETY: we know the structure of `ul_sys::ULVertex_2f_4ub_2f`
                // and we know that we match it with the format description.
                Ok(unsafe {
                    glium::VertexBuffer::new_raw(context, &buf, &BINDING_2F4UB2F, element_size)
                }?
                .into())
            }
            GliumDriverVertexBuffer::Format2f4ub2f2f28f(buf) => {
                let element_size = std::mem::size_of::<ul_sys::ULVertex_2f_4ub_2f_2f_28f>();

                // SAFETY: we know the structure of `ul_sys::ULVertex_2f_4ub_2f_2f_28f`
                // and we know that we match it with the format description.
                Ok(unsafe {
                    glium::VertexBuffer::new_raw(context, &buf, &BINDING_2F4UB2F2F28F, element_size)
                }?
                .into())
            }
        }
    }
}

/// A [`GpuDriver`] implemented for integrating with `glium`.
///
/// Since `glium` is a single threaded library, and [`GpuDriver`] need to implement
/// [`Send`] to be used in [`platform::set_gpu_driver`](crate::platform::set_gpu_driver),
/// we used a **Sender/Receiver** design here, where this is the sender and the receiver is
/// [`GliumGpuDriverReceiver`], which will handle all the sent commands from here,
/// and render them into textures.
pub struct GliumGpuDriverSender {
    next_texture_id: u32,
    next_render_buffer_id: u32,
    next_geometry_id: u32,
    sender: mpsc::Sender<GliumGpuCommand>,
}

impl GpuDriver for GliumGpuDriverSender {
    fn begin_synchronize(&mut self) {
        // unhandled
    }

    fn end_synchronize(&mut self) {
        // unhandled
    }

    fn next_texture_id(&mut self) -> u32 {
        self.next_texture_id += 1;
        self.next_texture_id
    }

    fn create_texture(&mut self, texture_id: u32, bitmap: OwnedBitmap) {
        self.sender
            .send(GliumGpuCommand::CreateTexture(texture_id, bitmap))
            .unwrap();
    }

    fn update_texture(&mut self, texture_id: u32, bitmap: OwnedBitmap) {
        self.sender
            .send(GliumGpuCommand::UpdateTexture(texture_id, bitmap))
            .unwrap();
    }

    fn destroy_texture(&mut self, texture_id: u32) {
        self.sender
            .send(GliumGpuCommand::DestroyTexture(texture_id))
            .unwrap();
    }

    fn next_render_buffer_id(&mut self) -> u32 {
        self.next_render_buffer_id += 1;
        self.next_render_buffer_id
    }

    fn create_render_buffer(&mut self, render_buffer_id: u32, render_buffer: RenderBuffer) {
        self.sender
            .send(GliumGpuCommand::CreateRenderBuffer(
                render_buffer_id,
                render_buffer,
            ))
            .unwrap();
    }

    fn destroy_render_buffer(&mut self, render_buffer_id: u32) {
        self.sender
            .send(GliumGpuCommand::DestroyRenderBuffer(render_buffer_id))
            .unwrap();
    }

    fn next_geometry_id(&mut self) -> u32 {
        self.next_geometry_id += 1;
        self.next_geometry_id
    }

    fn create_geometry(
        &mut self,
        geometry_id: u32,
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
    ) {
        let glium_vertex_buffer = match vertex_buffer.format {
            VertexBufferFormat::Format_2f_4ub_2f => {
                // SAFETY: since the source is `u8`, and we check that the `head`
                // and `tail` are empty, we make sure that all the bytes are
                // used in the format correctly.
                let (head, body, tail) = unsafe {
                    vertex_buffer
                        .buffer
                        .as_slice()
                        .align_to::<ul_sys::ULVertex_2f_4ub_2f>()
                };
                assert!(head.is_empty());
                assert!(tail.is_empty());

                GliumDriverVertexBuffer::Format2f4ub2f(body.to_vec())
            }
            VertexBufferFormat::Format_2f_4ub_2f_2f_28f => {
                // SAFETY: since the source is `u8`, and we check that the `head`
                // and `tail` are empty, we make sure that all the bytes are
                // used in the format correctly.
                let (head, body, tail) = unsafe {
                    vertex_buffer
                        .buffer
                        .as_slice()
                        .align_to::<ul_sys::ULVertex_2f_4ub_2f_2f_28f>()
                };
                assert!(head.is_empty());
                assert!(tail.is_empty());

                GliumDriverVertexBuffer::Format2f4ub2f2f28f(body.to_vec())
            }
        };

        self.sender
            .send(GliumGpuCommand::CreateGeometry(
                geometry_id,
                glium_vertex_buffer,
                index_buffer,
            ))
            .unwrap();
    }

    fn update_geometry(
        &mut self,
        geometry_id: u32,
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
    ) {
        let glium_vertex_buffer = match vertex_buffer.format {
            VertexBufferFormat::Format_2f_4ub_2f => {
                // SAFETY: since the source is `u8`, and we check that the `head`
                // and `tail` are empty, we make sure that all the bytes are
                // used in the format correctly.
                let (head, body, tail) = unsafe {
                    vertex_buffer
                        .buffer
                        .as_slice()
                        .align_to::<ul_sys::ULVertex_2f_4ub_2f>()
                };
                assert!(head.is_empty());
                assert!(tail.is_empty());

                GliumDriverVertexBuffer::Format2f4ub2f(body.to_vec())
            }
            VertexBufferFormat::Format_2f_4ub_2f_2f_28f => {
                // SAFETY: since the source is `u8`, and we check that the `head`
                // and `tail` are empty, we make sure that all the bytes are
                // used in the format correctly.
                let (head, body, tail) = unsafe {
                    vertex_buffer
                        .buffer
                        .as_slice()
                        .align_to::<ul_sys::ULVertex_2f_4ub_2f_2f_28f>()
                };
                assert!(head.is_empty());
                assert!(tail.is_empty());

                GliumDriverVertexBuffer::Format2f4ub2f2f28f(body.to_vec())
            }
        };

        self.sender
            .send(GliumGpuCommand::UpdateGeometry(
                geometry_id,
                glium_vertex_buffer,
                index_buffer,
            ))
            .unwrap();
    }

    fn destroy_geometry(&mut self, geometry_id: u32) {
        self.sender
            .send(GliumGpuCommand::DestroyGeometry(geometry_id))
            .unwrap();
    }

    fn update_command_list(&mut self, command_list: Vec<GpuCommand>) {
        self.sender
            .send(GliumGpuCommand::UpdateCommandList(command_list))
            .unwrap();
    }
}

struct GluimContextWrapper {
    context: Rc<Context>,
}

impl Facade for GluimContextWrapper {
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

/// The receiver part of [`GliumGpuDriverSender`].
///
/// Since `glium` is a single threaded library, and [`GpuDriver`] need to implement
/// [`Send`] to be used in [`platform::set_gpu_driver`](crate::platform::set_gpu_driver),
/// we used a **Sender/Receiver** design here, where [`GliumGpuDriverSender`]
/// is the sender and the receiver is this struct, when calling [`GliumGpuDriverReceiver::render`],
/// we will render all the commands we get from [`GliumGpuDriverSender`] into textures
/// which can be obtained by [`GliumGpuDriverReceiver::get_texture`].
pub struct GliumGpuDriverReceiver {
    /// receiver for the commands from the sender
    receiver: mpsc::Receiver<GliumGpuCommand>,
    /// glium context
    context: GluimContextWrapper,

    /// create a small texture, which will be used when
    /// the gpu driver doesn't set a texture for a draw call
    empty_texture: EitherTexture,
    /// map for (id -> texture), and storing the `render_buffer` id if applicable.
    texture_map: HashMap<u32, (EitherTexture, Option<u32>)>,
    /// map for (id -> render_buffer metadata), the render_buffer itself is a texture
    /// stored in the `texture_map`, we only create a framebuffer when drawing.
    render_buffer_map: HashMap<u32, RenderBuffer>,
    /// map for (id -> (vertex_buffer, index_buffer)).
    geometry_map: HashMap<u32, (VertexBufferAny, glium::IndexBuffer<u32>)>,

    /// Shader program for path rendering commands.
    path_program: Program,
    /// Shader program for fill rendering commands.
    fill_program: Program,
}

impl GliumGpuDriverReceiver {
    fn new(
        receiver: mpsc::Receiver<GliumGpuCommand>,
        context: &Rc<Context>,
    ) -> Result<Self, GliumGpuDriverError> {
        let context = GluimContextWrapper {
            context: context.clone(),
        };
        let empty_texture = EitherTexture::Regular2d(Texture2d::empty(&context, 1, 1)?);

        let texture_map = HashMap::new();
        let render_buffer_map = HashMap::new();
        let geometry_map = HashMap::new();

        let path_program = program!(&context,
        150 => {
            vertex: include_str!("./shaders/v2f_c4f_t2f_vert.glsl"),
            fragment: include_str!("./shaders/path_frag.glsl")
        })?;
        let fill_program = program!(&context,
        150 => {
            vertex: include_str!("./shaders/v2f_c4f_t2f_t2f_d28f_vert.glsl"),
            fragment: include_str!("./shaders/fill_frag.glsl")
        })?;

        Ok(GliumGpuDriverReceiver {
            receiver,
            context,
            empty_texture,
            texture_map,
            render_buffer_map,
            geometry_map,

            path_program,
            fill_program,
        })
    }

    /// helper function to create a texture based on bitmap
    fn create_texture(&self, bitmap: &OwnedBitmap) -> Result<EitherTexture, GliumGpuDriverError> {
        if bitmap.is_empty() {
            Texture2d::empty(&self.context, bitmap.width(), bitmap.height())
                .map_err(|e| e.into())
                .map(EitherTexture::Regular2d)
        } else {
            // since its not empty, it should have a valid pixels.
            let bitmap_pixels = bitmap.pixels().unwrap();

            match bitmap.format() {
                BitmapFormat::A8Unorm => {
                    let img = RawImage2d {
                        data: Cow::Borrowed(bitmap_pixels),
                        width: bitmap.width(),
                        height: bitmap.height(),
                        format: ClientFormat::U8,
                    };

                    Texture2d::with_format(
                        &self.context,
                        img,
                        UncompressedFloatFormat::U8,
                        MipmapsOption::NoMipmap,
                    )
                    .map_err(|e| e.into())
                    .map(EitherTexture::Regular2d)
                }
                BitmapFormat::Bgra8UnormSrgb => {
                    // FIXME: the number of pixels sometimes may not be `width * height * 4`
                    // because the bitmap will have padding for each row.
                    // Normally, this is fixable by using `UNPACK_ROW_LENGTH` in OpenGL,
                    // but glium doesn't support it for now

                    let expected_row_bytes = bitmap.width() * 4;
                    let data = if bitmap.row_bytes() != expected_row_bytes {
                        let mut new_data = Vec::with_capacity(
                            bitmap.height() as usize * expected_row_bytes as usize,
                        );
                        for row in bitmap_pixels.chunks(bitmap.row_bytes() as usize) {
                            new_data.extend_from_slice(&row[..expected_row_bytes as usize]);
                        }
                        Cow::Owned(new_data)
                    } else {
                        Cow::Borrowed(bitmap_pixels)
                    };

                    let img = RawImage2d {
                        data,
                        width: bitmap.width(),
                        height: bitmap.height(),
                        format: ClientFormat::U8U8U8U8,
                    };

                    SrgbTexture2d::with_format(
                        &self.context,
                        img,
                        glium::texture::SrgbFormat::U8U8U8U8,
                        MipmapsOption::NoMipmap,
                    )
                    .map_err(|e| e.into())
                    .map(EitherTexture::Srgb2d)
                }
            }
        }
    }
}

impl GliumGpuDriverReceiver {
    /// Fetch `glium` texture by id, this id can be obtained from the current
    /// `render_target` of a `view` by [`View::render_target`](crate::view::View::render_target).
    ///
    /// Example:
    /// ```no_run,ignore
    /// let render_target = view.render_target().unwrap();
    /// let texture = receiver.get_texture(&render_target.texture_id);
    /// ```
    pub fn get_texture(&self, id: &u32) -> Option<&EitherTexture> {
        self.texture_map.get(id).map(|(t, _)| t)
    }

    /// Flushes and renders all pending GPU commands recieved from [`GliumGpuDriverSender`],
    /// which will be generated when calling [`Renderer::render`](crate::renderer::Renderer::render).
    ///
    /// **Note that this must be called for rendering to actually occure, as using**
    /// **[`platform::set_gpu_driver`](crate::platform::set_gpu_driver) alone**
    /// **with [`GliumGpuDriverSender`] is not enough.**
    pub fn render(&mut self) -> Result<(), GliumGpuDriverError> {
        while let Ok(cmd) = self.receiver.try_recv() {
            match cmd {
                GliumGpuCommand::CreateTexture(id, bitmap) => {
                    let t = self.create_texture(&bitmap)?;
                    self.texture_map.insert(id, (t, None));
                }
                GliumGpuCommand::UpdateTexture(id, bitmap) => {
                    assert!(self.texture_map.contains_key(&id));

                    let t = self.create_texture(&bitmap)?;

                    let entry = self.texture_map.get_mut(&id).unwrap();
                    entry.0 = t;
                }
                GliumGpuCommand::DestroyTexture(id) => {
                    assert!(self.texture_map.contains_key(&id));
                    self.texture_map.remove(&id);
                }
                GliumGpuCommand::CreateRenderBuffer(id, render_buffer) => {
                    let entry = self.texture_map.get_mut(&render_buffer.texture_id).unwrap();
                    entry.1 = Some(id);
                    // make sure same texture sizes
                    assert!(entry.0.width() == render_buffer.width);
                    assert!(entry.0.height() == render_buffer.height);

                    self.render_buffer_map.insert(id, render_buffer);
                }
                GliumGpuCommand::DestroyRenderBuffer(id) => {
                    assert!(self.render_buffer_map.contains_key(&id));
                    let render_buffer = self.render_buffer_map.remove(&id).unwrap();
                    if let Some(entry) = self.texture_map.get_mut(&render_buffer.texture_id) {
                        entry.1 = None;
                    }
                }
                GliumGpuCommand::CreateGeometry(id, vert, index) => {
                    let index_buffer = glium::IndexBuffer::new(
                        &self.context,
                        glium::index::PrimitiveType::TrianglesList,
                        &index.buffer,
                    )?;

                    self.geometry_map.insert(
                        id,
                        (vert.into_glium_vertex_buffer(&self.context)?, index_buffer),
                    );
                }
                GliumGpuCommand::UpdateGeometry(id, vert, index) => {
                    assert!(self.geometry_map.contains_key(&id));

                    let index_buffer = glium::IndexBuffer::new(
                        &self.context,
                        glium::index::PrimitiveType::TrianglesList,
                        &index.buffer,
                    )?;

                    *self.geometry_map.get_mut(&id).unwrap() =
                        (vert.into_glium_vertex_buffer(&self.context)?, index_buffer);
                }
                GliumGpuCommand::DestroyGeometry(id) => {
                    assert!(self.geometry_map.contains_key(&id));
                    self.geometry_map.remove(&id);
                }
                GliumGpuCommand::UpdateCommandList(cmd_list) => {
                    for cmd in cmd_list {
                        match cmd {
                            GpuCommand::ClearRenderBuffer {
                                render_buffer_id: id,
                            } => {
                                assert!(self.render_buffer_map.contains_key(&id));
                                let render_buffer = self.render_buffer_map.get(&id).unwrap();

                                // TODO: add support
                                assert!(!render_buffer.has_stencil_buffer);
                                assert!(!render_buffer.has_depth_buffer);

                                let t = self.texture_map.get(&render_buffer.texture_id).unwrap();

                                let mut frame_buffer = SimpleFrameBuffer::new(&self.context, &t.0)?;

                                frame_buffer.clear(
                                    None,
                                    Some((0.0, 0.0, 0.0, 0.0)),
                                    false,
                                    None,
                                    None,
                                );
                            }
                            GpuCommand::DrawGeometry {
                                gpu_state,
                                geometry_id,
                                indices_count,
                                indices_offset,
                            } => {
                                assert!(self.geometry_map.contains_key(&geometry_id));
                                let (vertex_buffer, index_buffer) =
                                    self.geometry_map.get(&geometry_id).unwrap();

                                assert!(self
                                    .render_buffer_map
                                    .contains_key(&gpu_state.render_buffer_id));
                                let render_buffer = self
                                    .render_buffer_map
                                    .get(&gpu_state.render_buffer_id)
                                    .unwrap();

                                // TODO: add support
                                assert!(!render_buffer.has_stencil_buffer);
                                assert!(!render_buffer.has_depth_buffer);

                                let index_buffer_slice = index_buffer
                                    .slice(
                                        indices_offset as usize
                                            ..(indices_offset as usize + indices_count as usize),
                                    )
                                    .ok_or(GliumGpuDriverError::DrawIndexOutOfRange {
                                        index_buffer_size: index_buffer.len(),
                                        draw_index_offset: indices_offset,
                                        draw_index_size: indices_count,
                                    })?;

                                let (t, _) =
                                    self.texture_map.get(&render_buffer.texture_id).unwrap();

                                let mut frame_buffer = SimpleFrameBuffer::new(&self.context, t)?;

                                let used_program = match gpu_state.shader_type {
                                    ShaderType::Fill => &self.fill_program,
                                    ShaderType::FillPath => &self.path_program,
                                };

                                let scalar_data =
                                    UniformBuffer::new(&self.context, gpu_state.uniform_scalar)?;
                                let vector_data =
                                    UniformBuffer::new(&self.context, gpu_state.uniform_vector)?;
                                let clip_data = UniformBuffer::new(&self.context, gpu_state.clip)?;

                                // Orthographic Projection matrix applied to
                                // the `transformation` matrix.
                                let orth_projection_matrix = [
                                    [2.0 / gpu_state.viewport_width as f32, 0.0, 0.0, 0.0],
                                    [0.0, 2.0 / gpu_state.viewport_height as f32, 0.0, 0.0],
                                    [0.0, 0.0, -0.000002, 0.0],
                                    [-1.0, -1.0, 0.818183, 1.0],
                                ];
                                // trasform matrix to project matrix
                                let mut transformation = [
                                    [0., 0., 0., 0.],
                                    [0., 0., 0., 0.],
                                    [0., 0., 0., 0.],
                                    [0., 0., 0., 0.],
                                ];

                                // multiply matrices
                                #[allow(clippy::needless_range_loop)]
                                for i in 0..4 {
                                    for j in 0..4 {
                                        for k in 0..4 {
                                            transformation[i][j] += gpu_state.transform[i * 4 + k]
                                                * orth_projection_matrix[k][j];
                                        }
                                    }
                                }

                                // we use the supplied texture if it exists, or
                                // an empty texture if it doesn't.
                                let texture1 = if let Some(id) = gpu_state.texture_1_id {
                                    let (t, _) = self.texture_map.get(&id).unwrap();
                                    t
                                } else {
                                    &self.empty_texture
                                };
                                let texture2 = if let Some(id) = gpu_state.texture_2_id {
                                    let (t, _) = self.texture_map.get(&id).unwrap();
                                    t
                                } else {
                                    &self.empty_texture
                                };
                                let texture3 = if let Some(id) = gpu_state.texture_3_id {
                                    let (t, _) = self.texture_map.get(&id).unwrap();
                                    t
                                } else {
                                    &self.empty_texture
                                };

                                let uniforms = uniform! {
                                    // TODO: state time
                                    State: [0.0, gpu_state.viewport_width as f32, gpu_state.viewport_height as f32, 1.0],
                                    Transform: transformation,
                                    Scalar: &scalar_data,
                                    Vector: &vector_data,
                                    ClipSize: gpu_state.clip_size,
                                    Clip: &clip_data,
                                    Texture1: texture1.sampled(),
                                    Texture2: texture2.sampled(),
                                    Texture3: texture3.sampled(),
                                };

                                let params = DrawParameters {
                                    viewport: Some(glium::Rect {
                                        left: 0,
                                        bottom: 0,
                                        width: gpu_state.viewport_width,
                                        height: gpu_state.viewport_height,
                                    }),
                                    scissor: if gpu_state.enable_scissor {
                                        Some(glium::Rect {
                                            left: gpu_state.scissor_rect.left as u32,
                                            bottom: gpu_state.scissor_rect.top as u32,
                                            width: (gpu_state.scissor_rect.right
                                                - gpu_state.scissor_rect.left)
                                                as u32,
                                            height: (gpu_state.scissor_rect.bottom
                                                - gpu_state.scissor_rect.top)
                                                as u32,
                                        })
                                    } else {
                                        None
                                    },
                                    blend: if gpu_state.enable_blend {
                                        Blend::alpha_blending()
                                    } else {
                                        Blend::default()
                                    },
                                    ..DrawParameters::default()
                                };

                                frame_buffer.draw(
                                    vertex_buffer,
                                    index_buffer_slice,
                                    used_program,
                                    &uniforms,
                                    &params,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
