using psu_rebirth.DataTypes.Reflection;
using psu_rebirth.DataTypes.Reflection.Expressions;
using psu_rebirth.DataTypes.Reflection.Statements;
using System.Collections.Generic;
using System.Globalization;
using Terminal.Gui;
using Terminal.Gui.Trees;

namespace psu_cli {
    public class TreeVisitor : IVisitor {
        private TreeView view;
        public TreeNode currentNode = new TreeNode("main");
        public TreeVisitor(TreeView view) {
            this.view = view;
        }

        public void visit(ReflectionNode node) => node.visit(this);

        public void visit(NodeBody body) {
            foreach (NodeStatement node in body.body)
                node.visit(this);
        }

        public void visit(NodeDoStatement statement) {
            TreeNode newCurrent = new TreeNode("do") 
                {Tag = statement};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            statement.body.visit(this);

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeBreakStatement statement) {
            currentNode.Children.Add(new TreeNode("break") { Tag = statement });
        }
        
        public void visit(NodeContinueStatement statement) {
            currentNode.Children.Add(new TreeNode("continue") { Tag = statement });
        }

        public void visit(NodeReturnStatement statement) {
            TreeNode newCurrent = new TreeNode("return") {Tag = statement};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            statement.values.ForEach((NodeExpression value) => { value.visit(this); });

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeIfStatement statement) {
            TreeNode newCurrent = new TreeNode("if") {Tag = statement};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;
            
            statement.condition.visit(this);

            TreeNode thenNode = new TreeNode("then");
            currentNode = thenNode;
            statement.conditionBody.visit(this);
            newCurrent.Children.Add(thenNode);
            currentNode = newCurrent;
            
            foreach (var nodeStatement in statement.elseIfBodies)
            {
                var elseIfStat = (NodeIfStatement) nodeStatement;
                TreeNode elseIfNode = new TreeNode("elseif") {Tag = elseIfStat};
                currentNode = elseIfNode;
            
                elseIfStat.condition.visit(this);

                TreeNode elseIfBody = new TreeNode("then");
                currentNode = elseIfBody;
                elseIfStat.conditionBody.visit(this);
                elseIfNode.Children.Add(elseIfBody);
                currentNode = elseIfNode;
                
                newCurrent.Children.Add(currentNode);
                currentNode = newCurrent;
            }

            if (statement.elseBody is NodeBody)
            {
                TreeNode elseNode = new TreeNode("else");
                currentNode = elseNode;
                statement.elseBody.visit(this);
                newCurrent.Children.Add(elseNode);
                currentNode = newCurrent;
            }
            
            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeForStatement statement) {
            TreeNode newCurrent = new TreeNode($"{(statement is GenericForStatement ? "generic" : "numeric")} for stat") 
                { Tag = statement };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;
            
            statement.vars.ForEach((LocalData e) => { currentNode.Children.Add(new TreeNode(e.name.value) { Tag = e }); });
            switch (statement)
            {
                case GenericForStatement genericStatement:
                    currentNode = new TreeNode("in");
                    newCurrent.Children.Add(currentNode);
                    genericStatement.generatorList.ForEach((NodeExpression e) => { e.visit(this); });
                    currentNode = newCurrent;
                    break;
                case NumericForStatement numericStatement:
                    currentNode = new TreeNode("=");
                    newCurrent.Children.Add(currentNode);
                    numericStatement.rangeList.ForEach((NodeExpression e) => { e.visit(this); });
                    currentNode = newCurrent;
                    break;
            }

            TreeNode block = new TreeNode("do") 
                {Tag = statement.body};
            currentNode = block;
            statement.body.visit(this);

            newCurrent.Children.Add(currentNode);
            currentNode = newCurrent;

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }
        public void visit(NodeWhileStatement statement) {
            TreeNode newCurrent = new TreeNode("while") 
                { Tag = statement };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;
            
            statement.condition.visit(this);

            TreeNode block = new TreeNode("do") 
                {Tag = statement.body};
            currentNode = block;
            statement.body.visit(this);
            newCurrent.Children.Add(currentNode);
            
            currentNode = newCurrent;
            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeLocalDeclarationStatement statement) {
            TreeNode newCurrent = new TreeNode("local declaration") { Tag = statement };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            statement.locals.ForEach((LocalData e) => { currentNode.Children.Add(new TreeNode(e.name.value) { Tag = e }); });
            currentNode.Children.Add(new TreeNode("="));
            statement.values.ForEach((NodeExpression e) => { e.visit(this); });

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeAssignmentStatement statement) {
            TreeNode newCurrent = new TreeNode("assignment") { Tag = statement };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            statement.variables.ForEach((NodeExpression e) => {e.visit(this);});
            currentNode.Children.Add(new TreeNode("="));
            statement.values.ForEach((NodeExpression e) => { e.visit(this); });

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        private readonly Dictionary<CompoundOp, string> compoundLookup = new Dictionary<CompoundOp, string>() {
            {CompoundOp.Add, "+="},
            {CompoundOp.Sub, "-="},
            {CompoundOp.Mul, "*="},
            {CompoundOp.Div, "/="},
            {CompoundOp.Mod, "%="},
            {CompoundOp.Pow, "^="},
            {CompoundOp.Concat, "..="}
        };
        public void visit(NodeCompoundAssignmentStatement statement) {
            TreeNode newCurrent = new TreeNode("compound assignment") { Tag = statement };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            statement.variable.visit(this);
            currentNode.Children.Add(new TreeNode(compoundLookup[statement.operation]));
            statement.value.visit(this);

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeSoftStatement statement) {
            statement.expression.visit(this);
        }

        public void visit(NodeRepeatStatement statement) {
            TreeNode newCurrent = new TreeNode("repeat") {Tag = statement};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            TreeNode bodyNode = new TreeNode("body");
            currentNode = bodyNode;
            statement.body.visit(this);
            newCurrent.Children.Add(bodyNode);
            currentNode = newCurrent;
            
            TreeNode untilNode = new TreeNode("until");
            currentNode = untilNode;
            statement.condition.visit(this);
            newCurrent.Children.Add(untilNode);
            currentNode = newCurrent;

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeNilExpression expression) {
            var node = new TreeNode("nil") {Tag = expression};
            currentNode.Children.Add(node);
        }

        public void visit(NodeVarargExpression expression) {
            var node = new TreeNode("...") {Tag = expression};
            currentNode.Children.Add(node);
        }

        public void visit(NodeBooleanExpression expression) {
            var node = new TreeNode(expression.value.ToString()) {Tag = expression};
            currentNode.Children.Add(node);
        }

        public void visit(NodeStringExpression expression) {
            var delimiter = expression.delimiter is DelimiterType.ShortString ? "'"
                    : expression.delimiter is DelimiterType.String ? "\""
                        : "[[";
            
            var node = new TreeNode($"{delimiter}{expression.value}{(expression.delimiter == DelimiterType.LongString ? "]]" : delimiter)}")
                {Tag = expression};
            currentNode.Children.Add(node);
        }

        public void visit(NodeNumberExpression expression) {
            var node = new TreeNode(expression.raw.ToString(CultureInfo.InvariantCulture)) 
                { Tag = expression };
            currentNode.Children.Add(node);
        }

        private readonly Dictionary<UnaryOperation, string> unaryLookup = new Dictionary<UnaryOperation, string>() {
            { UnaryOperation.Len, "#" },
            { UnaryOperation.Not, "not" },
            { UnaryOperation.Minus, "-" }
        };
        public void visit(NodeUnaryExpression expression) {
            TreeNode newCurrent = new TreeNode("unary expression") 
                {Tag = expression};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            currentNode.Children.Add(new TreeNode(unaryLookup[expression.operation]));
            expression.expression.visit(this);

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        private readonly Dictionary<BinaryOperation, string> binaryLookup = new Dictionary<BinaryOperation, string>() {
            { BinaryOperation.Add, "+" },
            { BinaryOperation.Sub, "-" },
            { BinaryOperation.Mul, "*" },
            { BinaryOperation.Div, "/" },
            { BinaryOperation.Mod, "%" },
            { BinaryOperation.Pow, "^" },
            { BinaryOperation.CompareGt, ">" },
            { BinaryOperation.CompareLt, "<" },
            { BinaryOperation.CompareGe, ">=" },
            { BinaryOperation.CompareLe, "<=" },
            { BinaryOperation.CompareEq, "==" },
            { BinaryOperation.CompareNe, "~=" },
            { BinaryOperation.And, "and" },
            { BinaryOperation.Or, "or" }
        };
        public void visit(NodeBinaryExpression expression) {
            TreeNode newCurrent = new TreeNode("binary expression") 
                {Tag = expression};

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            expression.leftExpression.visit(this);
            currentNode.Children.Add(new TreeNode("operation: " + binaryLookup[expression.operation]) { Tag = expression.operation });
            expression.rightExpression.visit(this);

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeCallExpression expression) {
            TreeNode newCurrent = new TreeNode("call expression") { Tag = expression };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            expression.function.visit(this);
            var argsNode = new TreeNode("arguments");
            currentNode = argsNode;
            foreach (NodeExpression arg in expression.arguments)
                arg.visit(this);
            currentNode = newCurrent;
            
            newCurrent.Children.Add(argsNode);
            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeGroupExpression expression) {
            TreeNode newCurrent = new TreeNode("group expression") { Tag = expression };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            expression.expression.visit(this);

            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeTableExpression expression) {
            TreeNode newCurrent = new TreeNode("tree expression") { Tag = expression };

            TreeNode oldCurrent = currentNode;
            currentNode = newCurrent;

            var pairsNode = new TreeNode("pairs");
            currentNode = pairsNode;
            foreach (KeyValuePair<NodeExpression, NodeExpression> pair in expression.pairs) {
                var pairNode = new TreeNode("pair");
                currentNode = pairNode;

                pair.Key.visit(this);
                pair.Value.visit(this);
                pairsNode.Children.Add(pairNode);
                currentNode = pairsNode;
            }
            currentNode = newCurrent;

            newCurrent.Children.Add(pairsNode);
            oldCurrent.Children.Add(currentNode);
            currentNode = oldCurrent;
        }

        public void visit(NodeGlobalExpression expression) {
            var node = new TreeNode(expression.name.value) {Tag = expression};
            currentNode.Children.Add(node);
        }

        public void visit(NodeLocalExpression expression) {
            var node = new TreeNode(expression.local.name.value) {Tag = expression};
            currentNode.Children.Add(node);
        }
    }
}
