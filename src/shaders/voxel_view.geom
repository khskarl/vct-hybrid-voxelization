// Credits: Solomon Medland
// https://stackoverflow.com/questions/28375338/cube-using-single-gl-triangle-strip

#version 450

in VSOUT{
	vec4 position;
	vec3 w_position;
	vec4 color;
} v_in[];

out vec3 v_color;
out vec3 vw_position;

uniform sampler3D volume;
uniform mat4 mvp;
uniform int resolution;

layout (points) in;
layout(triangle_strip, max_vertices = 12) out;
void main() {
	float voxel_size = (1.0 / float(resolution)) * 1.00;

	v_color = v_in[0].color.rgb;
	vw_position = v_in[0].w_position;

	vec4 center = v_in[0].position;

	if(v_in[0].color.a < 0.001)
		return;

	vec4 dx = mvp[0] * voxel_size;
	vec4 dy = mvp[1] * voxel_size;
	vec4 dz = mvp[2] * voxel_size;

	vec4 p1 = center;
	vec4 p2 = center + dx;
	vec4 p3 = center + dy;
	vec4 p4 = p2 + dy;
	vec4 p5 = p1 + dz;
	vec4 p6 = p2 + dz;
	vec4 p7 = p3 + dz;
	vec4 p8 = p4 + dz;

	gl_Position = p4;
	EmitVertex();

	gl_Position = p3;
	EmitVertex();

	gl_Position = p2;
	EmitVertex();

	gl_Position = p1;
	EmitVertex();

	gl_Position = p5;
	EmitVertex();

	gl_Position = p3;
	EmitVertex();

	gl_Position = p7;
	EmitVertex();

	gl_Position = p4;
	EmitVertex();

	gl_Position = p8;
	EmitVertex();

	gl_Position = p2;
	EmitVertex();

	gl_Position = p6;
	EmitVertex();

	gl_Position = p5;
	EmitVertex();

	gl_Position = p8;
	EmitVertex();

	gl_Position = p7;
	EmitVertex();
}