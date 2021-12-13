using psu_rebirth.Lexer;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Expressions {
    [MoonSharpUserData]
    public class NodeNumberExpression : NodeExpression {
        public double value;
        public string raw;
        public LexicalType numberType;
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
