struct Params {
    width: u32,
};


// Input to the shader. The length of the array is determined by what buffer is bound.
//
// Out of bounds accesses
@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var<storage, read> input_vals: array<f32>;
// Output of the shader.
@group(0) @binding(2)
var<storage, read_write> output_vals: array<f32>;
@group(0) @binding(3)
var<storage, read> expr_starts: array<u32>;
@group(0) @binding(4)
var<storage, read> expr_parts_1: array<u32>;
@group(0) @binding(5)
var<storage, read> expr_parts_2: array<u32>;


// Ideal workgroup size depends on the hardware, the workload, and other factors. However, it should
// _generally_ be a multiple of 64. Common sizes are 64x1x1, 256x1x1; or 8x8x1, 16x16x1 for 2D workloads.
@compute @workgroup_size(256)
fn evalExpr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // While compute invocations are 3d, we're only using one dimension.
    let x = global_id.x;
    let y = global_id.y;

    let index = y * params.width + x;

    // Because we're using a workgroup size of 64, if the input size isn't a multiple of 64,
    // we will have some "extra" invocations. This is fine, but we should tell them to stop
    // to avoid out-of-bounds accesses.
    let array_length = arrayLength(&input_vals);
    if index >= array_length {
        return;
    }

    // Do the multiply by two and write to the output.
    output_vals[index] = eval_expr(index);
}

fn get_expr_part(index: u32) -> u32 {
    let expr_parts_1_length = arrayLength(&expr_parts_1);
    if index >= expr_parts_1_length {
        return expr_parts_2[index - expr_parts_1_length];
    }
    return expr_parts_1[index];
}

fn eval_expr(expr_index: u32) -> f32 {
    var i = expr_starts[expr_index];
    var current_roll: u32 = 0;
    var sum: f32 = 0.0;
    var current_max: f32 = -1.0;
    while current_roll < 5 {
        let part: u32 = get_expr_part(i);

        var val: f32 = 1.0;
        if !part_is_win(part) {
            val = input_vals[get_var(part)];
        }

        if part_is_inverse(part) {
            val = 1.0 - val;
        }

        current_max = max(current_max, val);

        if part_is_end(part) {
            sum += roll_weight(current_roll) * current_max;

            current_max = -1.0;
            current_roll += 1;
        }
        i += 1;
    }
    return sum;
}

fn roll_weight(roll: u32) -> f32 {
    if roll == 0 {
        return 1.0 / 16.0;
    } else if roll == 1 {
        return 4.0 / 16.0;
    } else if roll == 2 {
        return 6.0 / 16.0;
    } else if roll == 3 {
        return 4.0 / 16.0;
    } else if roll == 4 {
        return 1.0 / 16.0;
    }
    return 0.0;
}

fn part_is_end(part: u32) -> bool {
    return (part & (1 << 31)) != 0;
}

fn part_is_inverse(part: u32) -> bool {
    return (part & (1 << 30)) != 0;
}

fn part_is_win(part: u32) -> bool {
    return (part & (1 << 29)) != 0;
}

fn get_var(part: u32) -> u32 {
    return part & 0x1FFFFFFF;
}