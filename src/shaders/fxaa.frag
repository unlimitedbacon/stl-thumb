#version 150

uniform vec2 resolution;
uniform sampler2D tex;
uniform int enabled;

in vec2 v_tex_coords;
out vec4 fragColor;

#define FXAA_REDUCE_MIN   (1.0/ 128.0)
#define FXAA_REDUCE_MUL   (1.0 / 8.0)
#define FXAA_SPAN_MAX     8.0

vec4 fxaa(sampler2D tex, vec2 fragCoord, vec2 resolution,
            vec2 v_rgbNW, vec2 v_rgbNE, 
            vec2 v_rgbSW, vec2 v_rgbSE, 
            vec2 v_rgbM) {
    vec4 color;

    vec2 inverseVP = vec2(1.0 / resolution.x, 1.0 / resolution.y);
    vec3 rgbNW = texture(tex, v_rgbNW).xyz;
    vec3 rgbNE = texture(tex, v_rgbNE).xyz;
    vec3 rgbSW = texture(tex, v_rgbSW).xyz;
    vec3 rgbSE = texture(tex, v_rgbSE).xyz;
    vec4 texColor = texture(tex, v_rgbM);
    vec3 rgbM  = texColor.xyz;

    vec3 luma = vec3(0.299, 0.587, 0.114);
    vec4 luma4 = vec4(0.299, 0.587, 0.114, 0.0);

    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);
    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    
    vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));
    
    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) *
                          (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
              max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
              dir * rcpDirMin)) * inverseVP;
    
    vec2 coordAA = fragCoord * inverseVP + dir * (1.0 / 3.0 - 0.5);
    vec2 coordAB = fragCoord * inverseVP + dir * (2.0 / 3.0 - 0.5);
    float alphaAA = texture(tex, coordAA).a;
    float alphaAB = texture(tex, coordAB).a;
    float alphaATotal = alphaAA + alphaAB;
    float weightAA = alphaAA / alphaATotal;
    float weightAB = alphaAB / alphaATotal;
    vec4 rgbA = vec4(
        texture(tex, coordAA).rgb*weightAA + texture(tex, coordAB).rgb*weightAB,
        0.5 * (alphaAA + alphaAB)
    );

    vec2 coordBC = fragCoord * inverseVP + dir * -0.5;
    vec2 coordBD = fragCoord * inverseVP + dir * 0.5;
    float alphaBC = texture(tex, coordBC).a;
    float alphaBD = texture(tex, coordBD).a;
    float alphaBTotal = alphaAA + alphaAB + alphaBC + alphaBD;
    float weightBA = alphaAA / alphaBTotal;
    float weightBB = alphaAB / alphaBTotal;
    float weightBC = alphaBC / alphaBTotal;
    float weightBD = alphaBD / alphaBTotal;
    vec4 rgbB = vec4(
        texture(tex, coordAA).rgb*weightBA + texture(tex, coordAB).rgb*weightBB + texture(tex, coordBC).rgb*weightBC + texture(tex, coordBD).rgb*weightBD,
        0.25 * (alphaAA + alphaAB + alphaBC + alphaBD)
    );

    float lumaB = dot(rgbB, luma4);
    if ((lumaB < lumaMin) || (lumaB > lumaMax))
        color = rgbA;
    else
        color = rgbB;
    return color;
}

void main() {
    vec2 fragCoord = v_tex_coords * resolution; 
    vec4 color;
    if (enabled != 0) {
        vec2 inverseVP = 1.0 / resolution.xy;
        vec2 v_rgbNW = (fragCoord + vec2(-1.0, -1.0)) * inverseVP;
        vec2 v_rgbNE = (fragCoord + vec2(1.0, -1.0)) * inverseVP;
        vec2 v_rgbSW = (fragCoord + vec2(-1.0, 1.0)) * inverseVP;
        vec2 v_rgbSE = (fragCoord + vec2(1.0, 1.0)) * inverseVP;
        vec2 v_rgbM = vec2(fragCoord * inverseVP);
        color = fxaa(tex, fragCoord, resolution, v_rgbNW, v_rgbNE, v_rgbSW,
                     v_rgbSE, v_rgbM);
    } else {
        color = texture(tex, v_tex_coords);
    }
    fragColor = color;
}
