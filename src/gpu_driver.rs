//! Ultralight custom gpu driver.
//!
//! Ultralight allows to create a custom GPU driver to receive low level GPU commands
//! and render them on custom Textures, which can be used to integrate to your
//! game/application seamlessly.
//!
//! There is an example `C++` implementation for `OpenGL`, `DirectX11`, `DirectX12`
//! and `Metal` in the [`AppCore`](https://github.com/ultralight-ux/AppCore) repository.
//!
//! This library also have a custom GPU driver for [`glium`].

#[cfg(feature = "glium")]
#[cfg_attr(docsrs, doc(cfg(feature = "glium")))]
pub mod glium;

use std::slice;

use crate::{
    bitmap::{Bitmap, OwnedBitmap},
    platform::GPUDRIVER,
    rect::Rect,
};

#[derive(Debug)]
/// RenderBuffer description. (See [`GpuDriver::create_render_buffer`]).
pub struct RenderBuffer {
    /// The backing texture id for this render buffer.
    pub texture_id: u32,
    /// The width of the backing texture.
    pub width: u32,
    /// The height of the backing texture.
    pub height: u32,
    /// Does the backing texture contain stencil buffer. (currently unused always false).
    pub has_stencil_buffer: bool,
    /// Does the backing texture contain depth buffer. (currently unused always false).
    pub has_depth_buffer: bool,
}

impl From<ul_sys::ULRenderBuffer> for RenderBuffer {
    fn from(rb: ul_sys::ULRenderBuffer) -> Self {
        RenderBuffer {
            texture_id: rb.texture_id,
            width: rb.width,
            height: rb.height,
            has_stencil_buffer: rb.has_stencil_buffer,
            has_depth_buffer: rb.has_depth_buffer,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
/// Vertex buffer format types
pub enum VertexBufferFormat {
    /// Vertex format type for path vertices.
    Format_2f_4ub_2f = ul_sys::ULVertexBufferFormat_kVertexBufferFormat_2f_4ub_2f as isize,
    /// Vertex format type for quad vertices.
    Format_2f_4ub_2f_2f_28f =
        ul_sys::ULVertexBufferFormat_kVertexBufferFormat_2f_4ub_2f_2f_28f as isize,
}

impl TryFrom<ul_sys::ULVertexBufferFormat> for VertexBufferFormat {
    type Error = ();

    fn try_from(vbf: ul_sys::ULVertexBufferFormat) -> Result<Self, Self::Error> {
        match vbf {
            ul_sys::ULVertexBufferFormat_kVertexBufferFormat_2f_4ub_2f => {
                Ok(VertexBufferFormat::Format_2f_4ub_2f)
            }
            ul_sys::ULVertexBufferFormat_kVertexBufferFormat_2f_4ub_2f_2f_28f => {
                Ok(VertexBufferFormat::Format_2f_4ub_2f_2f_28f)
            }
            _ => Err(()),
        }
    }
}

// TODO: passing raw `[u8]` is not safe, maybe we should transmute them to
//       a specific format? like what we did in `glium` gpu_driver.
/// Vertex buffer, the buffer is used for `quad` or `path` rendering based on
/// the `format`. (See [`GpuDriver::create_geometry`]).
pub struct VertexBuffer {
    /// The format of the raw data. Either path or quad vertices.
    pub format: VertexBufferFormat,
    /// The raw vertex buffer data.
    pub buffer: Vec<u8>,
}

impl TryFrom<ul_sys::ULVertexBuffer> for VertexBuffer {
    type Error = ();

    fn try_from(vb: ul_sys::ULVertexBuffer) -> Result<Self, Self::Error> {
        if vb.data.is_null() {
            return Err(());
        }
        let format = VertexBufferFormat::try_from(vb.format)?;
        let buffer = unsafe { slice::from_raw_parts(vb.data, vb.size as usize) };
        Ok(VertexBuffer {
            format,
            buffer: buffer.to_vec(),
        })
    }
}

/// Index buffer. (See [`GpuDriver::create_geometry`]).
pub struct IndexBuffer {
    pub buffer: Vec<u32>,
}

impl From<ul_sys::ULIndexBuffer> for IndexBuffer {
    fn from(vb: ul_sys::ULIndexBuffer) -> Self {
        assert!(vb.size % 4 == 0);
        assert!(!vb.data.is_null());
        let index_slice = unsafe { slice::from_raw_parts(vb.data as _, vb.size as usize / 4) };
        IndexBuffer {
            buffer: index_slice.to_vec(),
        }
    }
}

// helper macro to convert arrays
macro_rules! from_ul_arr {
    ($arr:expr, $from:ident) => {
        [
            $arr[0].$from,
            $arr[1].$from,
            $arr[2].$from,
            $arr[3].$from,
            $arr[4].$from,
            $arr[5].$from,
            $arr[6].$from,
            $arr[7].$from,
        ]
    };
    (mat $arr:expr, $from:ident) => {
        [
            from_ul_arr!(mat $arr[0].$from),
            from_ul_arr!(mat $arr[1].$from),
            from_ul_arr!(mat $arr[2].$from),
            from_ul_arr!(mat $arr[3].$from),
            from_ul_arr!(mat $arr[4].$from),
            from_ul_arr!(mat $arr[5].$from),
            from_ul_arr!(mat $arr[6].$from),
            from_ul_arr!(mat $arr[7].$from),
        ]
    };
    (mat $arr: expr) => {
        [
            [$arr[0], $arr[1], $arr[2], $arr[3]],
            [$arr[4], $arr[5], $arr[6], $arr[7]],
            [$arr[8], $arr[9], $arr[10], $arr[11]],
            [$arr[12], $arr[13], $arr[14], $arr[15]],
        ]
    };
}

#[derive(Debug, Clone)]
/// Shader types, used by [`GpuState::shader_type`]
///
/// Each of these correspond to a vertex/pixel shader pair to be used.
/// You can find stock shader code for these in the `shaders` folder of the
/// [`AppCore`](https://github.com/ultralight-ux/AppCore) repo and also in
/// the [`glium`] custom `gpu_driver` implementation here.
pub enum ShaderType {
    /// Shader for the quad geometry.
    Fill = ul_sys::ULShaderType_kShaderType_Fill as isize,
    /// Shader for the path geometry.
    FillPath = ul_sys::ULShaderType_kShaderType_FillPath as isize,
}

impl TryFrom<ul_sys::ULShaderType> for ShaderType {
    type Error = ();

    fn try_from(st: ul_sys::ULShaderType) -> Result<Self, Self::Error> {
        match st {
            ul_sys::ULShaderType_kShaderType_Fill => Ok(ShaderType::Fill),
            ul_sys::ULShaderType_kShaderType_FillPath => Ok(ShaderType::FillPath),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
/// The GPU state description to be used when handling draw command.
/// (See [`GpuCommand::DrawGeometry`]).
pub struct GpuState {
    /// Viewport width in pixels.
    pub viewport_width: u32,
    /// Viewport height in pixels.
    pub viewport_height: u32,
    /// transformation matrix.
    ///
    /// you should multiply this with the screen-space orthographic projection
    /// matrix then pass to the vertex shader.
    pub transform: [f32; 16],
    /// Whether or not we should enable texturing for the current draw command.
    pub enable_texturing: bool,
    /// Whether or not we should enable blending for the current draw command.
    /// If blending is disabled, any drawn pixels should overwrite existing.
    /// This is mainly used so we can modify alpha values of the RenderBuffer
    /// during scissored clears.
    pub enable_blend: bool,
    /// The vertex/pixel shader program pair to use for the current draw command.
    pub shader_type: ShaderType,
    /// The render buffer to use for the current draw command.
    pub render_buffer_id: u32,
    /// The texture id to bind to slot #1.
    pub texture_1_id: Option<u32>,
    /// The texture id to bind to slot #2.
    pub texture_2_id: Option<u32>,
    /// The texture id to bind to slot #3.
    pub texture_3_id: Option<u32>,
    /// 8 scalar values to be passed to the shader as uniforms.
    pub uniform_scalar: [f32; 8],
    /// 8 vector values to be passed to the shader as uniforms.
    pub uniform_vector: [[f32; 4]; 8],
    /// clip size to be passed to the shader as uniforms.
    pub clip_size: u8,
    /// 8 clip matrices to be passed to the shader as uniforms.
    pub clip: [[[f32; 4]; 4]; 8],
    /// Whether or not scissor testing should be used for the current draw command.
    pub enable_scissor: bool,
    /// The scissor rect to use for scissor testing (units in pixels)
    pub scissor_rect: Rect<i32>,
}

impl TryFrom<ul_sys::ULGPUState> for GpuState {
    type Error = ();

    fn try_from(gs: ul_sys::ULGPUState) -> Result<Self, Self::Error> {
        Ok(GpuState {
            viewport_width: gs.viewport_width,
            viewport_height: gs.viewport_height,
            transform: gs.transform.data,
            enable_texturing: gs.enable_texturing,
            enable_blend: gs.enable_blend,
            shader_type: ShaderType::try_from(gs.shader_type as u32)?,
            render_buffer_id: gs.render_buffer_id,
            texture_1_id: if gs.texture_1_id == 0 {
                None
            } else {
                Some(gs.texture_1_id)
            },
            texture_2_id: if gs.texture_2_id == 0 {
                None
            } else {
                Some(gs.texture_2_id)
            },
            texture_3_id: if gs.texture_3_id == 0 {
                None
            } else {
                Some(gs.texture_3_id)
            },
            uniform_scalar: gs.uniform_scalar,
            uniform_vector: from_ul_arr!(gs.uniform_vector, value),
            clip_size: gs.clip_size,
            clip: from_ul_arr!(mat gs.clip, data),
            enable_scissor: gs.enable_scissor,
            scissor_rect: Rect::from(gs.scissor_rect),
        })
    }
}

#[derive(Debug, Clone)]
/// The GPU command to be executed.
///
/// This describes a command to be executed on the GPU.
///
/// Commands are dispatched to the GPU driver asynchronously via
/// [`update_command_list`][GpuDriver::update_command_list],
/// the GPU driver should consume these commands and execute them at an appropriate time.
pub enum GpuCommand {
    /// Clear a specific render buffer, to be prepared for drawing.
    ClearRenderBuffer {
        /// The render buffer to clear.
        render_buffer_id: u32,
    },
    /// Performs a draw command.
    DrawGeometry {
        // `gpu_state` is boxed because its too large, and its not good
        // to have large difference in size between the two variants in enum.
        /// The GPU state to use for the draw command. (contain the `render_buffer_id`)
        gpu_state: Box<GpuState>,
        /// The geometry (vertex_buffer/index_buffer pair) to be used for the draw command.
        geometry_id: u32,
        /// The index offset to start drawing from in the `index_buffer`.
        indices_offset: u32,
        /// The number of indices to draw.
        indices_count: u32,
    },
}

impl TryFrom<ul_sys::ULCommand> for GpuCommand {
    type Error = ();

    fn try_from(gc: ul_sys::ULCommand) -> Result<Self, Self::Error> {
        match gc.command_type as u32 {
            ul_sys::ULCommandType_kCommandType_DrawGeometry => Ok(GpuCommand::DrawGeometry {
                gpu_state: Box::new(GpuState::try_from(gc.gpu_state)?),
                geometry_id: gc.geometry_id,
                indices_count: gc.indices_count,
                indices_offset: gc.indices_offset,
            }),
            ul_sys::ULCommandType_kCommandType_ClearRenderBuffer => {
                Ok(GpuCommand::ClearRenderBuffer {
                    render_buffer_id: gc.gpu_state.render_buffer_id,
                })
            }
            _ => Err(()),
        }
    }
}

// TODO: we should not return `0` in ids, should we enforce it?
/// `GpuDriver` trait, dispatches GPU calls to the native driver.
///
/// This is automatically provided for you when you use [`App::new`](crate::app::App),
/// `AppCore` provides platform-specific implementations of `GpuDriver` for each OS.
///
/// If you are using [`Renderer::create`](crate::renderer::Renderer::create),
/// you will need to provide your own implementation of this trait if you
/// have enabled the GPU renderer in the Config.
/// (See [`platform::set_gpu_driver`](crate::platform::set_gpu_driver)).
pub trait GpuDriver {
    /// Called before any commands are dispatched during a frame.
    ///
    /// Called before any state is updated during a call to
    /// [`Renderer::render`](crate::renderer::Renderer::render).
    /// This is a good time to prepare the GPU for any state updates.
    fn begin_synchronize(&mut self);
    /// Called after any commands are dispatched during a frame.
    ///
    /// Called after all state has been updated during a call to
    /// [`Renderer::render`](crate::renderer::Renderer::render).
    fn end_synchronize(&mut self);
    /// Get the next available texture ID. **DO NOT return `0` (reserved)**.
    ///
    /// This is used to generate a unique texture ID for each texture created by the library.
    /// The GPU driver implementation is responsible for mapping these IDs to a native ID.
    fn next_texture_id(&mut self) -> u32;
    /// Create a texture with a certain ID and optional bitmap.
    ///
    /// **NOTE**: If the Bitmap is empty [`OwnedBitmap::is_empty`],
    /// then a RTT Texture should be created instead.
    /// This will be used as a backing texture for a new RenderBuffer.
    ///
    /// Even if the bitmap is empty, it will still contain the `width` and `height`
    /// information, which can be used to know the size of the backing texture.
    fn create_texture(&mut self, texture_id: u32, bitmap: OwnedBitmap);
    /// Update an existing non-RTT texture with new bitmap data.
    fn update_texture(&mut self, texture_id: u32, bitmap: OwnedBitmap);
    /// Destroy a texture.
    fn destroy_texture(&mut self, texture_id: u32);
    /// Generate the next available render buffer ID. **DO NOT return `0` (reserved)**.
    fn next_render_buffer_id(&mut self) -> u32;
    /// Create a render buffer with certain ID and buffer description.
    fn create_render_buffer(&mut self, render_buffer_id: u32, render_buffer: RenderBuffer);
    /// Destroy a render buffer.
    fn destroy_render_buffer(&mut self, render_buffer_id: u32);
    /// Get the next available geometry ID. **DO NOT return `0`**.
    fn next_geometry_id(&mut self) -> u32;
    /// Create geometry with certain ID and vertex/index data.
    fn create_geometry(
        &mut self,
        geometry_id: u32,
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
    );
    /// Update existing geometry with new vertex/index data.
    fn update_geometry(
        &mut self,
        geometry_id: u32,
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
    );
    /// Destroy a geometry.
    fn destroy_geometry(&mut self, geometry_id: u32);
    /// Update command list (here you should render the commands).
    fn update_command_list(&mut self, command_list: Vec<GpuCommand>);
}

platform_set_interface_macro! {
    #[inline]
    pub(crate) set_gpu_driver<GpuDriver>(lib, gpu_driver -> GPUDRIVER) -> ulPlatformSetGPUDriver(ULGPUDriver) {
        begin_synchronize() -> () {}
        end_synchronize() -> () {}
        next_texture_id(() -> u32) -> () {}
        create_texture((texture_id: u32, ul_bitmap: ul_sys::ULBitmap)) -> ((texture_id: u32, bitmap: OwnedBitmap)) {
            let mut bitmap = Bitmap::from_raw(lib.clone(), ul_bitmap).unwrap();
            let bitmap = OwnedBitmap::from_bitmap(&mut bitmap).unwrap();
        }
        update_texture((texture_id: u32, ul_bitmap: ul_sys::ULBitmap)) -> ((texture_id: u32, bitmap: OwnedBitmap)) {
            let mut bitmap = Bitmap::from_raw(lib.clone(), ul_bitmap).unwrap();
            let bitmap = OwnedBitmap::from_bitmap(&mut bitmap).unwrap();
        }
        destroy_texture((texture_id: u32)) -> ((texture_id: u32)) {}
        next_render_buffer_id(() -> u32) -> () {}
        create_render_buffer((render_buffer_id: u32, ul_render_buffer: ul_sys::ULRenderBuffer))
            -> ((render_buffer_id: u32, render_buffer: RenderBuffer)) {
            let render_buffer = RenderBuffer::from(ul_render_buffer);
        }
        destroy_render_buffer((render_buffer_id: u32)) -> ((render_buffer_id: u32)) {}
        next_geometry_id(() -> u32) -> () {}
        create_geometry((geometry_id: u32, ul_vertex_buffer: ul_sys::ULVertexBuffer,
            ul_index_buffer: ul_sys::ULIndexBuffer)) -> ((geometry_id: u32, vertex_buffer: VertexBuffer, index_buffer: IndexBuffer)) {
            let vertex_buffer = VertexBuffer::try_from(ul_vertex_buffer).unwrap();
            let index_buffer = IndexBuffer::from(ul_index_buffer);
        }
        update_geometry((geometry_id: u32, ul_vertex_buffer: ul_sys::ULVertexBuffer,
            ul_index_buffer: ul_sys::ULIndexBuffer)) -> ((geometry_id: u32, vertex_buffer: VertexBuffer, index_buffer: IndexBuffer)) {
            let vertex_buffer = VertexBuffer::try_from(ul_vertex_buffer).unwrap();
            let index_buffer = IndexBuffer::from(ul_index_buffer);
        }
        destroy_geometry((geometry_id: u32)) -> ((geometry_id: u32)) {}
        update_command_list((ul_command_list: ul_sys::ULCommandList)) -> ((commands_list: Vec<GpuCommand>)) {
            assert!(!ul_command_list.commands.is_null());
            let commands_slice = slice::from_raw_parts(ul_command_list.commands, ul_command_list.size as usize);
            let commands_list = commands_slice.iter().map(|gc| GpuCommand::try_from(*gc).unwrap()).collect();
        }
    }
}
