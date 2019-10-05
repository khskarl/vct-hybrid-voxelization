#version 450 core

void main() {
	// out_color = v_color;
	// out_color = vec4(vw_position / float(resolution), 1.0);
	// out_color = vec4(v_color.xyz, 1.0);
	gl_FragColor = vec4( 1, 1, 1, 1 );
}
