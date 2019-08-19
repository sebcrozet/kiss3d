#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec3 local_light_position;
varying vec2 tex_coord_v;
varying vec3 normalInterp;
varying vec3 vertPos;

uniform vec3 color;
uniform sampler2D tex;
const vec3 specColor = vec3(0.4, 0.4, 0.4);

void main() {
  vec3 normal = normalize(normalInterp);
  vec3 lightDir = normalize(local_light_position - vertPos);

  float lambertian = max(dot(lightDir, normal), 0.0);
  float specular = 0.0;

  if(lambertian > 0.0) {
    vec3 viewDir = normalize(-vertPos);
    vec3 halfDir = normalize(lightDir + viewDir);
    float specAngle = max(dot(halfDir, normal), 0.0);
    specular = pow(specAngle, 30.0);
  }

  vec4 tex_color = texture2D(tex, tex_coord_v);
  gl_FragColor = tex_color * vec4(color / 3.0 +
                                  lambertian * color / 3.0 +
                                  specular * specColor / 3.0, 1.0);
}
