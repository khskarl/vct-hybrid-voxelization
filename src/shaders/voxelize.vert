#version 450 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out vec3 v_position;
out vec2 v_uv;
out vec3 v_normal;

out VSOUT {
	vec3 position;
	vec2 uv;
	vec3 normal;
} v_out;

uniform layout(rgba8, binding = 0) image3D u_voxel_diffuse;

uniform int u_width;
uniform int u_height;
uniform int u_depth;

void main() {
	v_out.uv = aTexCoord;
	v_out.normal = aNormal;

	uint i = gl_VertexID % u_width;
	uint j = (gl_VertexID / u_width) % u_height;
	uint k = (gl_VertexID / u_width / u_height) % u_depth;
	ivec3 coordinate = ivec3(i, j, k);

	gl_Position = vec4(coordinate - vec3(0.0, 2.0, 0.0), 1.0);

	v_out.position = coordinate;
	vec3 color = vec3(coordinate) / u_width;
	float dist = length(vec3(8.0, 8.0, 8.0) - vec3(coordinate));
	float alpha = dist < 8.0 ? 1.0 : 0.0;
	imageStore(u_voxel_diffuse, coordinate, vec4(color, alpha));



  // mat4 model_matrix = primitive_parameters.model_matrix;
  // mat3 normal_matrix = transpose(inverse(mat3(model_matrix)));

  // vertex_world_position = vec3(model_matrix * vec4(position, 1.0));
  // v_out. = normalize(normal_matrix * normal);
  // v_out.uv = aTexCoord;

  // gl_Position = vec4(vertex_world_position, 1.0f);
}
