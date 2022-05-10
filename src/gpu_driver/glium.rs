//! A custom [`GpuDriver`] implementation for the `glium` backend.

use std::{borrow::Cow, collections::HashMap, rc::Rc, sync::mpsc};

use glium::{
    backend::{Context, Facade},
    framebuffer::SimpleFrameBuffer,
    pixel_buffer::PixelBuffer,
    program,
    texture::{ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat},
    uniform,
    uniforms::UniformBuffer,
    vertex::VertexBufferAny,
    Blend, DrawParameters, Program, Surface, Texture2d,
};

use crate::{
    bitmap::{BitmapFormat, OwnedBitmap},
    gpu_driver::ShaderType,
};

use super::{GpuCommand, GpuDriver, IndexBuffer, RenderBuffer, VertexBuffer, VertexBufferFormat};

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
/// ```no_run
/// let (sender, mut receiver) = create_gpu_driver(&display);
/// Platform::set_gpu_driver(sender);
///
/// renderer.render(); // will dispatch and send all events to `reciever` from `ultralight`
/// receiver.render(); // will render all events received from `sender`
/// ```
pub fn create_gpu_driver<F: ?Sized>(facade: &F) -> (GliumGpuDriverSender, GliumGpuDriverReceiver)
where
    F: Facade,
{
    let (sender, receiver) = mpsc::channel();
    (
        GliumGpuDriverSender {
            next_texture_id: 0,
            next_render_buffer_id: 0,
            next_geometry_id: 0,
            sender,
        },
        GliumGpuDriverReceiver::new(receiver, facade.get_context()),
    )
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
    fn into_glium_vertex_buffer<F: ?Sized>(&self, context: &F) -> VertexBufferAny
    where
        F: Facade,
    {
        match self {
            GliumDriverVertexBuffer::Format2f4ub2f(buf) => {
                let format = Cow::Owned(vec![
                    (
                        Cow::Borrowed("in_Position"),
                        0,
                        glium::vertex::AttributeType::F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Color"),
                        2 * ::std::mem::size_of::<f32>(),
                        glium::vertex::AttributeType::U8U8U8U8,
                        true,
                    ),
                    (
                        Cow::Borrowed("in_TexCoord"),
                        2 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32,
                        false,
                    ),
                ]);
                let element_size = std::mem::size_of::<ul_sys::ULVertex_2f_4ub_2f>();

                // SAFETY: we know the structure of `ul_sys::ULVertex_2f_4ub_2f`
                // and we know that we match it with the format description.
                unsafe { glium::VertexBuffer::new_raw(context, &buf, format, element_size) }
                    .unwrap()
                    .into()
            }
            GliumDriverVertexBuffer::Format2f4ub2f2f28f(buf) => {
                let format = Cow::Owned(vec![
                    (
                        Cow::Borrowed("in_Position"),
                        0,
                        glium::vertex::AttributeType::F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Color"),
                        2 * ::std::mem::size_of::<f32>(),
                        glium::vertex::AttributeType::U8U8U8U8,
                        true,
                    ),
                    (
                        Cow::Borrowed("in_TexCoord"),
                        2 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_ObjCoord"),
                        4 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data0"),
                        6 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data1"),
                        10 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data2"),
                        14 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data3"),
                        18 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data4"),
                        22 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data5"),
                        26 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                    (
                        Cow::Borrowed("in_Data6"),
                        30 * ::std::mem::size_of::<f32>() + 4 * ::std::mem::size_of::<u8>(),
                        glium::vertex::AttributeType::F32F32F32F32,
                        false,
                    ),
                ]);
                let element_size = std::mem::size_of::<ul_sys::ULVertex_2f_4ub_2f_2f_28f>();

                // SAFETY: we know the structure of `ul_sys::ULVertex_2f_4ub_2f_2f_28f`
                // and we know that we match it with the format description.
                unsafe { glium::VertexBuffer::new_raw(context, &buf, format, element_size) }
                    .unwrap()
                    .into()
            }
        }
    }
}

/// A [`GpuDriver`] implemented for integrating with `glium`.
///
/// Since `glium` is a single threaded library, and [`GpuDriver`] need to implement
/// [`Send`] to be used in [`Platform::set_gpu_driver`](crate::platform::Platform::set_gpu_driver),
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
/// [`Send`] to be used in [`Platform::set_gpu_driver`](crate::platform::Platform::set_gpu_driver),
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
    empty_texture: Texture2d,
    /// map for (id -> texture), and storing the `render_buffer` id if applicable.
    texture_map: HashMap<u32, (Texture2d, Option<u32>)>,
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
    fn new(receiver: mpsc::Receiver<GliumGpuCommand>, context: &Rc<Context>) -> Self {
        let context = GluimContextWrapper {
            context: context.clone(),
        };
        let empty_texture = Texture2d::empty(&context, 1, 1).unwrap();

        let texture_map = HashMap::new();
        let render_buffer_map = HashMap::new();
        let geometry_map = HashMap::new();

        let path_program = program!(&context,
        150 => {
            vertex: include_str!("./shaders/v2f_c4f_t2f_vert.glsl"),
            fragment: include_str!("./shaders/path_frag.glsl")
        })
        .unwrap();
        let fill_program = program!(&context,
        150 => {
            vertex: include_str!("./shaders/v2f_c4f_t2f_t2f_d28f_vert.glsl"),
            fragment: include_str!("./shaders/fill_frag.glsl")
        })
        .unwrap();

        GliumGpuDriverReceiver {
            receiver,
            context,
            empty_texture,
            texture_map,
            render_buffer_map,
            geometry_map,

            path_program,
            fill_program,
        }
    }

    /// helper function to create a texture based on bitmap
    fn create_texture(&self, bitmap: &OwnedBitmap) -> Texture2d {
        if bitmap.is_empty() {
            Texture2d::empty(&self.context, bitmap.width(), bitmap.height()).unwrap()
        } else {
            match bitmap.format() {
                BitmapFormat::A8Unorm => {
                    let img = RawImage2d {
                        data: Cow::Borrowed(bitmap.pixels()),
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
                    .unwrap()
                }
                BitmapFormat::Bgra8UnormSrgb => {
                    // glium doesn't support BGRA when creating new texture,
                    // so we have to upload manually
                    let t = Texture2d::empty_with_format(
                        &self.context,
                        UncompressedFloatFormat::U8U8U8U8,
                        MipmapsOption::NoMipmap,
                        bitmap.width(),
                        bitmap.height(),
                    )
                    .unwrap();

                    let pixels: Vec<_> = bitmap
                        .pixels()
                        .chunks(4)
                        .map(|c| (c[0], c[1], c[2], c[3]))
                        .collect();
                    let pixel_buffer = PixelBuffer::new_empty(&self.context, pixels.len());
                    pixel_buffer.write(pixels.as_slice());

                    t.main_level().raw_upload_from_pixel_buffer_inverted(
                        pixel_buffer.slice(..).unwrap(),
                        0..bitmap.width(),
                        0..bitmap.height(),
                        0..1,
                    );

                    t
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
    /// ```no_run
    /// let render_target = view.render_target().unwrap();
    /// let texture = receiver.get_texture(&render_target.texture_id);
    /// ```
    pub fn get_texture(&self, id: &u32) -> Option<&Texture2d> {
        self.texture_map.get(id).map(|(t, _)| t)
    }

    /// Flushes and renders all pending GPU commands recieved from [`GliumGpuDriverSender`],
    /// which will be generated when calling [`Renderer::render`](crate::renderer::Renderer::render).
    ///
    /// **Note that this must be called for rendering to actually occure, as using**
    /// **[`Platform::set_gpu_driver`](crate::platform::Platform::set_gpu_driver) alone**
    /// **with [`GliumGpuDriverSender`] is not enough.**
    pub fn render(&mut self) {
        while let Ok(cmd) = self.receiver.try_recv() {
            match cmd {
                GliumGpuCommand::CreateTexture(id, bitmap) => {
                    let t = self.create_texture(&bitmap);
                    self.texture_map.insert(id, (t, None));
                }
                GliumGpuCommand::UpdateTexture(id, bitmap) => {
                    assert!(self.texture_map.contains_key(&id));

                    let t = self.create_texture(&bitmap);

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
                    )
                    .unwrap();

                    self.geometry_map.insert(
                        id,
                        (vert.into_glium_vertex_buffer(&self.context), index_buffer),
                    );
                }
                GliumGpuCommand::UpdateGeometry(id, vert, index) => {
                    assert!(self.geometry_map.contains_key(&id));

                    let index_buffer = glium::IndexBuffer::new(
                        &self.context,
                        glium::index::PrimitiveType::TrianglesList,
                        &index.buffer,
                    )
                    .unwrap();

                    *self.geometry_map.get_mut(&id).unwrap() =
                        (vert.into_glium_vertex_buffer(&self.context), index_buffer);
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

                                let mut frame_buffer =
                                    SimpleFrameBuffer::new(&self.context, &t.0).unwrap();

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

                                let (t, _) =
                                    self.texture_map.get(&render_buffer.texture_id).unwrap();

                                let mut frame_buffer =
                                    SimpleFrameBuffer::new(&self.context, t).unwrap();

                                let used_program = match gpu_state.shader_type {
                                    ShaderType::Fill => &self.fill_program,
                                    ShaderType::FillPath => &self.path_program,
                                };

                                let scalar_data =
                                    UniformBuffer::new(&self.context, gpu_state.uniform_scalar)
                                        .unwrap();
                                let vector_data =
                                    UniformBuffer::new(&self.context, gpu_state.uniform_vector)
                                        .unwrap();
                                let clip_data =
                                    UniformBuffer::new(&self.context, gpu_state.clip).unwrap();

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
                                            bottom: gpu_state.scissor_rect.bottom as u32,
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

                                frame_buffer
                                    .draw(
                                        vertex_buffer,
                                        index_buffer
                                            .slice(
                                                indices_offset as usize
                                                    ..(indices_offset as usize
                                                        + indices_count as usize),
                                            )
                                            .unwrap(),
                                        used_program,
                                        &uniforms,
                                        &params,
                                    )
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
