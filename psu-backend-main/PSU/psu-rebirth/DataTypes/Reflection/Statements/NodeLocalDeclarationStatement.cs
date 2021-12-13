using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Statements {
    [MoonSharpUserData]
    public class NodeLocalDeclarationStatement : NodeStatement {
        public List<LocalData> locals = new List<LocalData>();
        public List<NodeExpression> values;
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
