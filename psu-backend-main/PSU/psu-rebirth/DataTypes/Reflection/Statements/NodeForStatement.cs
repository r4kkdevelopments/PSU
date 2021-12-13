using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Statements {
    [MoonSharpUserData]
    public class NodeForStatement : NodeStatement {
        public NodeBody body;
        public List<LocalData> vars = new List<LocalData>();
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }

    [MoonSharpUserData]
    public class GenericForStatement : NodeForStatement
    {
        public List<NodeExpression> generatorList = new List<NodeExpression>();
    }

    [MoonSharpUserData]
    public class NumericForStatement : NodeForStatement
    {
        public List<NodeExpression> rangeList = new List<NodeExpression>();
    }
}
