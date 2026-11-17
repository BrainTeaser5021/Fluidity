#version 140
uniform sampler2D sampler;
uniform float progress; // 0.0 (open) -> 1.0 (fully dissolved into background)
uniform vec2 resolution;

in vec2 texcoord0;
out vec4 fragColor;

void main() {
    // Sink UV coordinates downward with a radial liquid ripple
    float ripple = sin(texcoord0.x * 18.0 + progress * 10.0) * 0.025 * progress;
    float sink = progress * progress * 0.45;
    vec2 puddleUV = vec2(
        texcoord0.x + ripple * (1.0 - texcoord0.y),
        texcoord0.y - sink + ripple * 0.5
    );

    if (puddleUV.y < 0.0 || puddleUV.y > 1.0 || puddleUV.x < 0.0 || puddleUV.x > 1.0) {
        fragColor = vec4(0.0);
        return;
    }

    vec4 color = texture2D(sampler, puddleUV);
    // Smoothly fade from bottom-up and attenuate alpha into the wallpaper
    float puddleMask = smoothstep(progress * 1.15, progress * 0.35, 1.0 - texcoord0.y);
    float alpha = (1.0 - progress) * mix(0.45, 1.0, puddleMask);
    fragColor = vec4(color.rgb * alpha, color.a * alpha);
}
