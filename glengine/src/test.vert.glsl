#version 100
uniform mat4 tf;

attribute vec2 pos;
attribute vec4 col;
attribute vec2 texc;

varying vec4 v_col;
varying vec2 v_texc;

void main()
{
    v_col = col;
    v_texc = texc;
    gl_Position = tf * vec4(pos, 0.0, 1.0);
}
