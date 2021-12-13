using psu_rebirth.DataTypes.Reflection.Expressions;
using psu_rebirth.DataTypes.Reflection.Statements;

namespace psu_rebirth.DataTypes.Reflection {
    public interface IVisitor {
        void visit(ReflectionNode node);
        void visit(NodeBody body);

        void visit(NodeDoStatement statement);
        void visit(NodeBreakStatement statement);
        void visit(NodeContinueStatement statement);
        void visit(NodeReturnStatement statement);
        void visit(NodeIfStatement statement);
        void visit(NodeForStatement statement);
        void visit(NodeWhileStatement statement);
        void visit(NodeSoftStatement statement);
        void visit(NodeRepeatStatement statement);
        void visit(NodeAssignmentStatement statement);
        void visit(NodeCompoundAssignmentStatement statement);
        void visit(NodeLocalDeclarationStatement statement);

        void visit(NodeNilExpression expression);
        void visit(NodeVarargExpression expression);
        void visit(NodeBooleanExpression expression);
        void visit(NodeStringExpression expression);
        void visit(NodeNumberExpression expression);
        void visit(NodeUnaryExpression expression);
        void visit(NodeBinaryExpression expression);
        void visit(NodeCallExpression expression);
        void visit(NodeGroupExpression expression);
        void visit(NodeGlobalExpression expression);
        void visit(NodeLocalExpression expression);
        void visit(NodeTableExpression expression);
    }
}
