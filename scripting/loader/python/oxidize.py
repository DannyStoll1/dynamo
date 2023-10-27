# Convert SymPy expression to Rust string
def oxidize_expr(expr):
    if expr.is_Add:
        return " + ".join([f"({oxidize_expr(arg)})" for arg in expr.args])
    elif expr.is_Mul:
        return " * ".join([f"({oxidize_expr(arg)})" for arg in expr.args])
    elif expr.is_Pow:
        base, expo = expr.args
        if expo == -1:
            return f"({oxidize_expr(base)}).inv()"
        elif expo.is_Integer:
            return f"({oxidize_expr(base)}).powi({expo})"
        elif expo.is_Float:
            return f"({oxidize_expr(base)}).powf({expo})"
        else:
            return f"(Cplx::from({oxidize_expr(base)})).powc(({oxidize_expr(expo)}).into())"
    elif expr.is_Function:
        return f"({oxidize_expr(expr.args[0])}).{expr.func}()"
        # Add more functions as needed
    elif expr.is_Symbol:
        return str(expr)
    elif expr.is_Integer or expr.is_Float:
        return str(float(expr))
    elif expr.is_constant:
        return str(float(expr))
    else:
        raise Exception(f"Unsupported operation: {expr}")

def oxidize_expr_cplx(expr):
    s = oxidize_expr(expr)
    return f"Cplx::from({s})"


def oxidize_cse(cse):
    [scaffolds, outputs] = cse
    lines = [f"let {name} = {oxidize_expr(value)};" for (name, value) in scaffolds]
    lines.append("({})".format(", ".join(map(oxidize_expr, outputs))))

    return "\n".join(lines)


def oxidize_cse_cplx(cse):
    [scaffolds, outputs] = cse
    lines = [f"let {name} = {oxidize_expr(value)};" for (name, value) in scaffolds]
    lines.append("({})".format(", ".join(map(oxidize_expr_cplx, outputs))))

    return "\n".join(lines)


def oxidize_param_map(params_dict):
    lines = [f"let {name} = {oxidize_expr(value)};" for (name, value) in params_dict.items()]
    lines.append("Self::Param {{ {} }}".format(", ".join(params_dict.keys())))

    return "\n".join(lines)


def oxidize_param_map_cplx(params_dict):
    lines = [f"let {name} = {oxidize_expr_cplx(value)};" for (name, value) in params_dict.items()]
    lines.append("Self::Param {{ {} }}".format(", ".join(params_dict.keys())))

    return "\n".join(lines)


if __name__ == "__main__":
    from sympy import symbols, cse

# Define the variables
    x, y = symbols('x y')

# Define the SymPy expression
    expr0 = (x**2 + 3)/(x**2 - 1)
    expr1 = expr0.diff(x)

    print(oxidize_expr(expr0))

    cse_terms = cse([expr0, expr1], optimizations="basic")

    res = oxidize_cse(cse_terms)
    print(res)
