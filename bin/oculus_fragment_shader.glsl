#version 120
uniform sampler2D fbo_texture;
uniform float kappa_0;
uniform float kappa_1;
uniform float kappa_2;
uniform float kappa_3;
const vec2 LensCenterLeft = vec2(0.25, 0.5);
const vec2 LensCenterRight = vec2(0.75, 0.5);
uniform vec2 Scale;
uniform vec2 ScaleIn;

varying vec2 v_coord;
varying vec2 f_texcoord;

// Debugging
const vec4 red = vec4(1.0, 0.0, 0.0, 0.0);
const vec4 blue = vec4(0.0, 0.0, 1.0, 0.0);

void main()
{
    vec2 theta;
    float rSq;
    vec2 rvector;
    vec2 tc;
    bool left_eye;

    if (f_texcoord.x < 0.5) {
        left_eye = true;
    } else {
        left_eye = false;
    }

    if (left_eye) {
        theta = (f_texcoord - LensCenterLeft) * ScaleIn; 
    } else {
        theta = (f_texcoord - LensCenterRight) * ScaleIn; 
    }
    rSq = theta.x * theta.x + theta.y * theta.y;
    rvector = theta * (kappa_0 + kappa_1 * rSq + kappa_2 * rSq * rSq + kappa_3 * rSq * rSq * rSq);
    if (left_eye) {
        tc = LensCenterLeft + Scale * rvector;
    } else {
        tc = LensCenterRight + Scale * rvector;
    }

    //keep within bounds of texture 
    if ((left_eye && (tc.x < 0.0 || tc.x > 0.5)) ||   
        (!left_eye && (tc.x < 0.5 || tc.x > 1.0)) ||
        tc.y < 0.0 || tc.y > 1.0) {
        discard;
    }
/*

    //keep on either side of viewport
    if (left_eye && v_coord.x > 0.5) {
        discard;
    }
    else if (!left_eye && v_coord.x < 0.5) {
        discard;
    }
    */

    gl_FragColor = texture2D(fbo_texture, tc); 
    //gl_FragColor = red * f_texcoord.x + blue * (1.0 - f_texcoord.x);
}
