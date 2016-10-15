#version 100
uniform mat4 tf;

attribute vec2 pos;
attribute vec4 col;

varying vec4 v_col;

void main()
{
    v_col = col;
    gl_Position = tf * vec4(pos, 0.0, 1.0);
}
