                            #version 100

                            attribute vec2 position;
                            attribute vec2 i_tex_coords;

                            varying vec2 v_tex_coords;

                            void main() {
                                gl_Position = vec4(position, 0.0, 1.0);
                                v_tex_coords = i_tex_coords;
                            }
