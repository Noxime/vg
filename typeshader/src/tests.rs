// only required in this crate due to rust being rust
use crate as typeshader;
use crate::shader;

//const MAGENTA_SHADER: ProgramSource<(Attr<Vec4>, Attr<Vec2>), (Uni<Vec4>, Uni<Texture2D>)> = ...;

shader! {
    EMPTY = {
        vertex {
            glsl: "#version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) in vec2 a_pos;
            
            void main() {
                gl_Position = vec4(a_pos, 0.0, 1.0);
            }
            "
        },
        fragment {
            glsl: "#version 450
            #extension GL_ARB_separate_shader_objects : enable
            
            layout(location = 0) out vec4 color;
            
            void main() {
                color = vec4(1.0, 0.0, 1.0, 1.0);
            }
            "
        },
    };
    SHOW_TEX = {
        vertex {
            glsl: "#version 450
            #extension GL_ARB_separate_shader_objects : enable
            
            layout(binding = 0) uniform UniformBufferObject {
                mat4 model;
                mat4 view;
                mat4 proj;
            } ubo;

            layout(binding = 3) uniform UniformBufferObject2 {
                float time;
            } ubo2;

            layout(location = 0) in vec3 a_pos;
            layout(location = 1) in vec2 a_tex;
            
            layout(location = 0) out vec2 v_tex;
            
            void main() {
                gl_Position = vec4(a_pos, 1.0) * ubo.model * ubo.view * ubo.proj;
                v_tex = a_tex;
            }
            "
        },
        fragment {
            glsl: "#version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) in vec2 v_tex;
            
            layout(location = 0) out vec4 color;
            
            void main() {
                color = vec4(v_tex, 0.0, 1.0);
            }
            "
        },
    };
}

#[test]
fn empty() {
    
}
