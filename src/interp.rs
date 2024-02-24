use crate::parser::{Expr, Expr_::*, Program};
use std::collections::HashMap;

struct Props<'a> {
    pub pc: u16,
    pub ins: Vec<u8>,
    pub labels: HashMap<&'a str, u16>,
    pub line: usize,
}

pub fn interp<'a>(p: &'a Program) -> Vec<u8> {
    let mut props = Props {
        pc: 0,
        ins: vec![],
        labels: HashMap::new(),
        line: 0,
    };

    for expr in &p.statements {
        props.pc += 2;
        props.line += 1;
        interp_expr(&mut props, expr);
    }

    props.ins
}

fn interp_expr<'a>(props: &mut Props<'a>, expr: &'a Expr) {
    match expr.node {
        Clear => {
            // 00E0
            props.ins.extend(vec![0x00, 0xE0]);
        }
        AssignRegisterRegister(ref r1, ref r2) => {
            // 8xy0
            props.ins.extend(vec![0x80 + r1, r2 << 4]);
        }
        AssignRegisterInteger(ref r, ref int) => {
            // 6xnn
            props.ins.extend(vec![0x60 + r, *int]);
        }
        AssignIRegisterInteger(ref int) => {
            // Annn
            let high_byte = 0xD0 + ((int & 0xF00) >> 8);
            let low_byte = int & 0x0FF;
            props.ins.extend(vec![high_byte as u8, low_byte as u8]);
        }
        AssignIRegisterRegisterSprite(ref r) => {
            // Fx1E
            props.ins.extend(vec![0xF0 + r, 0x29]);
        }
        DeclareLabel(ref id) => {
            props.labels.insert(id, props.pc);
            props.pc -= 2;
        }
        DrawIRegister(ref r1, ref r2, ref int) => {
            // Dxyn
            props.ins.extend(vec![0xD0 + r1, (r2 << 4) + int]);
        }
        IncrementRegisterInteger(ref r, ref int) => {
            // 7xnn
            props.ins.extend(vec![0x70 + r, *int]);
        }
        GotoLabel(ref id) => {
            // 1nnn
            let Some(pc) = props.labels.get(id.as_str()) else {
                panic!("line {}: could not find label {:?}", props.line, id);
            };
            let high_byte = 0x10 + ((pc & 0xF00) >> 8);
            let low_byte = (pc + 510) & 0x0FF;
            props.ins.extend(vec![high_byte as u8, low_byte as u8]);
        }
    }
}
