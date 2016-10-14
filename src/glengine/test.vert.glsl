#version 100
uniform mat3 tf;

attribute vec2 pos;
attribute vec4 col;

varying vec4 v_col;

void main()
{
    v_col = col;
    gl_Position = (tf * vec3(pos, 1.0)).xyzz;
}
