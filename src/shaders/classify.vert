#version 450 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out VSOUT {
	vec3 w_position;
	vec3 w_normal;
	vec2 uv;
	int  id;
} v_out;

uniform mat4 pv;
uniform mat4 model;

layout(binding = 0, rgba8) uniform coherent image3D u_voxel_albedo;

vec3 to_voxel_space(vec3 pos) {
	return (pos + vec3(1.0)) * 0.5 * 64.0;
}

void main() {
	vec4 position = pv * model * vec4(aPosition, 1.0);
	gl_Position = position;

	v_out.w_position = to_voxel_space(position.xyz);
	v_out.w_normal = normalize(vec3(model * vec4(aNormal, 1.0)));
	v_out.uv = aTexCoord;
	v_out.id = gl_VertexID;

	// imageStore(u_voxel_albedo, coordinates, vec4(aNormal, 1.0));
}
