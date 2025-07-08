struct Params {
    width: u32,
};


// Input to the shader. The length of the array is determined by what buffer is bound.
//
// Out of bounds accesses
@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var<storage, read> input: array<f32>;
// Output of the shader.
@group(0) @binding(2)
var<storage, read_write> output: array<f32>;
@group(0) @binding(3)
var<storage, read> expr_parts: array<u32>;
@group(0) @binding(4)
var<storage, read> expr_starts: array<u32>;


// Ideal workgroup size depends on the hardware, the workload, and other factors. However, it should
// _generally_ be a multiple of 64. Common sizes are 64x1x1, 256x1x1; or 8x8x1, 16x16x1 for 2D workloads.
@compute @workgroup_size(256)
fn doubleMe(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // While compute invocations are 3d, we're only using one dimension.
    let x = global_id.x;
    let y = global_id.y;

    let index = y * params.width + x;

    // Because we're using a workgroup size of 64, if the input size isn't a multiple of 64,
    // we will have some "extra" invocations. This is fine, but we should tell them to stop
    // to avoid out-of-bounds accesses.
    let array_length = arrayLength(&input);
    if index >= array_length {
        return;
    }

    // Do the multiply by two and write to the output.
    output[index] = f32(expr_starts[index]);
}