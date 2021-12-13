using System;
using System.Diagnostics;
using System.Diagnostics.SymbolStore;
using System.Linq;
using System.Net;
using System.Runtime.CompilerServices;
using psu_rebirth.DataTypes.Reflection;

namespace psu_rebirth.Lexer {
    public class Lexer {
        private readonly string source;

        public Lexeme current;
        private Lexeme lookAhead;
        private uint lineNumber = 1;
        private uint lineOffset = 1; // Gotta add this >:(

        private void incrementLineNumber()
        {
            lineOffset = 1;
            lineNumber++;
        }
        public uint position;

        public static readonly string[] keywords = {
            "if", "then", "else", "elseif", "end",
            "for", "in", "and", "break", "while", "do",
            "local", "not", "or", "repeat",
            "until", "return", "true", "false", "nil",
            "function", "continue", "goto"
        };
            
        private static readonly string[] compounds = {
            "+=", "-=", "*=", "/=", "%=", "^=", "..="
        };  
        
        private static readonly char[] algorithmicOp = {
            '+', '-', '*', '/', '%', '^', '#'
        };

        private static readonly string[] conditional = {
            "==", "~=", "<", "<=", ">", ">="
        };

        private static readonly char[] symbols = {
            '.', ',', ';', ':', '(', ')', '{', '}', '[', ']', '='
        };
            

        public Lexer(string source) {
            this.source = source;

            // check if keywords and reserved enum section is the same length.
            Debug.Assert(
                (keywords.Length) == (uint)LexicalType.ReservedSectionEnd - (uint)LexicalType.ReservedSectionStart
            );

            next();
        }

        private static bool isWhitespace(char c) =>
            c is ' ' or '\t' or '\r' or '\n' or '\f' or '\v';
        private static bool isNewLine(char c) => c == '\n';
        private static bool isAlpha(char c) => (uint)(c - 'A') < 26 || (uint)(c - 'a') < 26;
        private static bool isDigit(char c) => (uint)(c - '0') < 10;
        private static bool isHex(char c) => (uint)(c - '0') < 10 || (uint)(c - 'A') < 6 || (uint)(c - 'a') < 6;
        private static bool isBit(char c) => c is '0' or '1';
        
        private char peek(uint offset = 0) => (position + offset) < (source.Length) ? source[(int) (position + offset)] : (char)LexicalType.Eof;
        private char read() => lineOffset++ < 0 ? (char) 0 :
            (position) < (source.Length) ? source[(int) position++] : (char) LexicalType.Eof;
        private void readSave(uint offset = 1)
        {
            for (uint i = 0; i < offset; i++)
            {
                current.data += read();
            }
        }
        
        private LexicalType readBin()
        {
            position += 2; // Assuming the caller has asserted this already (0b)
            while (isBit(peek()))
                readSave();
            
            return LexicalType.Binary;
        }
        
        private LexicalType readHex()
        {
            position += 2; // Assuming the caller has asserted this already (0x)
            // Exponent : Pp, gotta add
            while (isHex(peek()))
                readSave();

            return LexicalType.Hex;
        }
        
        // readNumeral, says to itself what it does. Todo - add exponent reading (0e0+ ...)
        private LexicalType readNumeral()
        {
            Debug.Assert(isDigit(peek()) || (peek() == '.' && isDigit(peek(1))));
            if (peek() == '0' && peek(1) == 'x')
                return readHex();
            if (peek() == '0' && peek(1) == 'b')
                return readBin();
            
            bool dot = peek() == '.';
            current.data = dot ? "0" : "";
            LexicalType type = LexicalType.Number;
            if (peek() == '.')
                readSave();
            
            Debug.Assert(!dot || isDigit(peek()));
            while (isDigit(peek()) || peek() == '.' || peek() is 'e' or 'E')
            {
                if (peek() is 'e' or 'E')
                {
                    type = LexicalType.Exponent;
                    if (peek(1) is '+' or '-')
                        readSave();
                }
                else if (peek() == '.' && dot == false)
                    dot = true;
                else if(peek() == '.' && dot)
                    Debug.Fail("Malformed number");
                readSave();
            }
            // Assert that the number is actually a number
            return type;
        }
        
        // Len( [(=*)[ | ](=*)] )
        private uint readSep() {
            uint count = 0;
            char s = peek();
            readSave();
            Debug.Assert(s is '[' or ']');
            
            while (peek() == '=') {
                count++;
                readSave();
            }
            
            Debug.Assert(peek() == s);
            readSave();
            return count;
        }

        private int readDecEsc()
        {
            int i = 0;
            int r = 0;
            for (i = 0; i < 3 && isDigit(peek()); i++)
            {
                r = 10 * r + peek() - '0';
                readSave();
            }

            current.data = current.data.Substring(0, current.data.Length - i);
            return r;
        }

        // ".*" | '.*', gotta add support for escaped characters
        private LexicalType readString()
        {
            char open = read();
            Debug.Assert(open is '\"' or '\'');
            while (peek() != (char) LexicalType.Eof && peek() != open)
                switch (peek())
                {
                    case (char)LexicalType.Eof:
                        Debug.Fail($"Unfinished string, got {(char)LexicalType.Eof}, expected {open}");
                        break;
                    case '\n':
                    case '\r':
                        Debug.Fail($"Unfinished string, got {peek()}, expected {open}");
                        break;
                    // Handle escaped characters
                    case '\\':
                        // read \ddd, \xD+, \uX+, \n, \t, ...
                        read();
                        int c;
                        switch (peek())
                        {
                            case 'a': c = '\a'; 
                                goto Save;
                            case 'b': c = '\b';
                                goto Save;
                            case 'f': c = '\f';
                                goto Save;
                            case 'n': c = '\n';
                                goto Save;
                            case 't': c = '\t';
                                goto Save;
                            case 'v': c = '\v';
                                goto Save;
                            case '\\': case '\'': case '\"':
                                readSave();
                                current.data = current.data[1..];
                                break;
                            default:
                                Debug.Assert(isDigit(peek()), "Invalid escaped sequence");
                                c = readDecEsc();
                                current.data += $"{(char)c}";
                                break;
                            Save:
                                read();
                                current.data += $"{(char)c}";
                                break;
                        }
                        
                        break;
                    default: 
                        readSave();
                        break;
                }
           
            Debug.Assert(read() == open);
            return open == '\'' ? LexicalType.ShortString : LexicalType.String;
        }
        
        // [=*[.*]=*], Long String, Todo - add better output for line number
        private LexicalType readLongString(uint sep)
        {
            while(true) {
                if (peek() is '\n' or '\r')
                {
                    incrementLineNumber();
                    current.data += '\n';
                } else
                    switch (peek())
                    {
                        // Handle escaped characters
                        case (char)LexicalType.Eof:
                            Debug.Fail("Malformed long string");
                            return LexicalType.Eof;
                        case ']':
                            if (sep == readSep())
                                goto EndPoint;
                            break;
                        default:
                            readSave();
                            break;
                    }
            } EndPoint:
            
            current.data = current.data.Substring(2 + (int)sep, current.data.Length - 4 - ((int)sep * 2));
            return LexicalType.LongString; // Could be a long comment, but we are assuming its a long string
        }
        
        private LexicalType lex()
        {
           // read whitespace
            for (;;)
            {
                // Whitespace and (Long)Comments
                while (true)
                {
                    if (isWhitespace(peek()))
                    {
                        if (isNewLine(read()))
                            incrementLineNumber();
                    }
                    else if (peek() == '-' && peek(1) == '-')
                    {
                        position += 2;
                        if (peek() == '[')
                        {
                            readLongString(readSep());
                            //return LexicalType.LongComment;
                        }

                        while (!isNewLine(peek()))
                            read();
                        incrementLineNumber();
                        //return LexicalType.Comment;
                    }
                    else
                    {
                        break;
                    }
                }

                
                // read tokens
                char c = peek();
                current.data = "";
                current.location.start = new Position() { line = lineNumber, column = lineOffset };
                
                if (c == (char) LexicalType.Eof)
                    return (LexicalType)read();
                
                if (isAlpha(c) || c == '_')
                {
                    // Identifiers and keywords
                    // [_, a-z, A-Z][_, 0-9, a-z, A-Z]+
                    readSave();
                    while (isAlpha(peek()) || isDigit(peek()) || peek() == '_')
                        readSave();
                    
                    return keywords.Contains(current.data)
                        ? Array.FindIndex(keywords, x => x == current.data)
                           + LexicalType.ReservedIf
                        : LexicalType.Name;
                }

                // Numbers or Digits, [0-9]+.[0-9]*, 0b, 0x, ...
                if (isDigit(c) || (c == '.' && isDigit(peek(1))))
                    return readNumeral();

                // Short or Long String
                if (c is '\"' or '\'')
                {
                    // (Short)String
                    return readString();
                }

                if (c == '[' && peek() is '=' or '[')
                {
                    // Long String
                    return readLongString(readSep());
                }

                // Either ., .., ..=, or ...
                if (c == '.')
                {
                    readSave();
                    if (peek() == '.')
                    {
                        readSave();
                        switch (peek())
                        {
                            case '.':
                                readSave();
                                return LexicalType.Vararg;
                            case '=':
                                readSave();
                                return LexicalType.CompoundConcat;
                            default:
                                return LexicalType.Concat;        
                        }
                    }
                }

                // +, -, *, /, ^, %, #, or any compound
                if (algorithmicOp.Contains(c))
                {
                    readSave();
                    if (peek() == '=')
                    {
                        string compound = $"{current.data}{peek()}";
                        if (compounds.Contains(compound))
                        {
                            readSave();
                            return Array.FindIndex(compounds, x => x == current.data) 
                                   + LexicalType.CompoundSectionStart;
                        }
                    }
                    
                    return Array.FindIndex(algorithmicOp, x => x == c)
                           + LexicalType.AlgorithmicSectionStart;
                    
                }

                //æ
                if (conditional.Contains($"{c}{peek(1)}"))
                {
                    readSave(2);
                    return Array.FindIndex(conditional, x => x == current.data) 
                           + LexicalType.BinopSectionStart;
                }

                // <, >
                if (conditional.Contains(c.ToString()))
                {
                    readSave();
                    return Array.FindIndex(conditional, x => x == current.data) 
                           + LexicalType.BinopSectionStart;
                }

                // Any other symbol, {, }, [, ], ., ,, ...
                if (symbols.Contains(c))
                {
                    readSave();
                    return Array.FindIndex(symbols, x => x == c) 
                           + LexicalType.SymbolSectionStart;
                }

            }
        }
        
        public bool testNext(LexicalType type) => glance() == type;

        public LexicalType glance()
        {
            string temp = current.data; 
            lookAhead.type = lex();
            lookAhead.data = current.data;
            current.data = temp;
            return lookAhead.type;
        }
        
        public LexicalType next()
        {
            current.location = new Location();
            current.type = lex();
            current.location.end = new Position() { line = lineNumber, column = lineOffset };
            return current.type;
        }
    }
}