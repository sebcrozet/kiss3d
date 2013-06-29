pub static vertex_src: &'static str =
   "#version 150                                                        \n\
    in vec3 position;                                                   \n\
    in vec3 color;                                                      \n\
    out vec3 Color;                                                     \n\
    uniform mat4 projection;                                            \n\
    uniform mat4 transform;                                             \n\
    void main() {                                                       \n\
        Color       = color;                                            \n\
        gl_Position = projection * transform * vec4(position, 1.0);     \n\
    }";

pub static fragment_src: &'static str =
   "#version 150                               \n\
    uniform vec3 color;                        \n\
    out vec4 outColor;                         \n\
    void main() {                              \n\
        outColor = vec4(color, 1.0);           \n\
    }";
