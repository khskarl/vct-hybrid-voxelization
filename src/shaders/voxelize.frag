#version 450 core

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform layout(rgba8, binding = 0) image3D u_voxel_diffuse;

void main() {
	ivec3 position = ivec3(gl_FragCoord.xy, gl_FragCoord.z * u_depth);

	vec3 color = vec3(position) / vec3(u_width, u_height, u_depth);
	// float dist = length(vec3(8.0, 8.0, 8.0) - vec3(position));
	// float alpha = dist < 8.0 ? 1.0 : 0.0;
	imageStore(u_voxel_diffuse, position, vec4(color, 1.0));

	// out_color = v_color;
	// out_color = vec4(vw_position / float(resolution), 1.0);
	// out_color = vec4(v_color.xyz, 1.0);
	// gl_FragColor = vec4( 1, 1, 1, 1 );
}
