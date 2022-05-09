#version 150

// Program Uniforms
uniform vec4 State;
uniform mat4 Transform;
uniform Scalar {
    float scalar[8];
};
uniform Clip {
    mat4 clip[8];
};
uniform Vector {
    vec4 vector[8];
};
uniform uint ClipSize;

// Uniform Accessor Functions
float Time() { return State[0]; }
float ScreenWidth() { return State[1]; }
float ScreenHeight() { return State[2]; }
float ScreenScale() { return State[3]; }
vec4 sRGBToLinear(vec4 val) { return vec4(val.xyz * (val.xyz * (val.xyz * 0.305306011 + 0.682171111) + 0.012522878), val.w); }

// Vertex Attributes
in vec2 in_Position;
in vec4 in_Color;
in vec2 in_TexCoord;
in vec2 in_ObjCoord;
in vec4 in_Data0;
in vec4 in_Data1;
in vec4 in_Data2;
in vec4 in_Data3;
in vec4 in_Data4;
in vec4 in_Data5;
in vec4 in_Data6;

// Out Params
out vec4 ex_Color;
out vec2 ex_TexCoord;
out vec4 ex_Data0;
out vec4 ex_Data1;
out vec4 ex_Data2;
out vec4 ex_Data3;
out vec4 ex_Data4;
out vec4 ex_Data5;
out vec4 ex_Data6;
out vec2 ex_ObjectCoord;
out vec2 ex_ScreenCoord;

void main(void)
{
  ex_ObjectCoord = in_ObjCoord;
  gl_Position = Transform * vec4(in_Position, 0.0, 1.0);
  ex_Color = in_Color;
  ex_TexCoord = in_TexCoord;
  ex_Data0 = in_Data0;
  ex_Data1 = in_Data1;
  ex_Data2 = in_Data2;
  ex_Data3 = in_Data3;
  ex_Data4 = in_Data4;
  ex_Data5 = in_Data5;
  ex_Data6 = in_Data6;
}

