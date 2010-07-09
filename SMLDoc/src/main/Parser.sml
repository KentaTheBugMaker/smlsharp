(**
 * @author YAMATODANI Kiyoshi
 * @version $Id: Parser.sml,v 1.11 2007/09/19 05:28:55 matsu Exp $
 *)
structure Parser : PARSER =
struct

  (***************************************************************************)

  structure AA = AnnotatedAst
  structure DGP = DocumentGenerationParameter

  structure MLLrVals = MLLrValsFun(structure Token = LrParser.Token)
  structure MLLexer = MLLexFun(structure Tokens = MLLrVals.Tokens)
  structure MLParser =
  JoinWithArg(structure ParserData = MLLrVals.ParserData
	      structure Lex = MLLexer
	      structure LrParser = LrParser)

  structure ParamPatternLrVals =
  ParamPatternLrValsFun(structure Token = LrParser.Token);
  structure ParamPatternLexer =
  ParamPatternLexFun(structure Tokens = ParamPatternLrVals.Tokens);
  structure ParamPatternParser =
  JoinWithArg(structure ParserData = ParamPatternLrVals.ParserData
	      structure Lex = ParamPatternLexer
	      structure LrParser = LrParser);

  (****************************************)

  structure SS = Substring

  (***************************************************************************)

  (**
   * parses a ML source file.
   * @params fileName
   * @param fileName the name of source file
   * @return (a list of AST, a list of docComments, a parserOperations)
   *)
  fun parseML parameter fileName =
      let

        exception EndOfParse

        type pos = int

        type lexarg =
             {
               comLevel : int ref,
               commonOperations : ParserUtil.PositionMap.operations,
               docComments : (string * int * int) list ref,
               error : (string * int * int) -> unit,
               stringBuf : string list ref,
               stringStart : pos ref,
               stringType : bool ref
             }

        val sourceStream = TextIO.openIn fileName
        val docComments = ref []
        val parserOperations = ParserUtil.PositionMap.create fileName

        fun onParseError arg =
            DGP.error parameter (#makeMessage parserOperations arg)

        val initialArg =
            {
              comLevel = ref 0,
              commonOperations = parserOperations,
              docComments = docComments,
              error = onParseError,
              stringBuf = ref nil : string list ref,
              stringStart = ref 0,
              stringType = ref true
            } : lexarg

        local
          val dummyEOF = MLLrVals.Tokens.EOF (0, 0)
          val dummySEMICOLON = MLLrVals.Tokens.SEMICOLON (0, 0)
        in
        fun oneParse lexer =
	    let 
	      val (nextToken, lexer') = MLParser.Stream.get lexer
	    in
	      if MLParser.sameToken(nextToken, dummyEOF)
              then raise EndOfParse
	      else
                if MLParser.sameToken(nextToken, dummySEMICOLON)
                then oneParse lexer'
	        else MLParser.parse(0, lexer, onParseError, ())
	    end
        end

        fun untilEOF lexer results =
            let val (ast, lexer') = oneParse lexer
            in untilEOF lexer' (ast :: results) end
              handle EndOfParse => List.rev results
                   | MLParser.ParseError => List.rev results

        fun getLine length = case TextIO.inputLine sourceStream of NONE => ""
								 | SOME s => s

        val asts =
            untilEOF (MLParser.makeLexer getLine initialArg) []
            handle e => (TextIO.closeIn sourceStream; raise e)
      in
        TextIO.closeIn sourceStream; (asts, !docComments, parserOperations)
      end

  (****************************************)

  (**
   * parses a doc comment.
   * @params parameter parserOperations (commentText, left, right)
   * @param parameter common parameter
   * @param parserOperations generated by parseML function for the ML source
   * @param commentText text of the doc comment
   * @param left left position of the doc comment in the source code
   * @param right right position of the doc comment in the source code
   * @return ((summary section, description section, tagBlocks), left, right)
   *)
  fun parseDocComment
      parameter
      (parserOperations : ParserUtil.PositionMap.operations)
      (docComment, left, right) =
      let
        open DocComment

        fun onParseError (message, leftInDocComment, rightInDocComment) =
            DGP.error
                parameter
                ((#makeMessage parserOperations)
                 (message, left + leftInDocComment, left + rightInDocComment))

        (**********)
        (* low-level I/O *)

        val sourceStream = TextIO.openString docComment

        (* the number of characters read from the sourceStream *)
        val readChars = ref 0

        (**
         *  If the line starts with whitespaces following an asterisk,
         * drops them.
         *)
        fun removeHeadingAsterisk line =
            let
              val substring = SS.extract (line, 0, NONE)
              val (_, trailer) = SS.splitl Char.isSpace substring
            in
              case SS.first trailer of
                SOME #"*" => SS.triml 1 trailer
              | _ => substring
            end

        (**
         *  reads characters until EOF is found. 
         * @return SOME(text, range) if some characters have been read from
         *      the stream.
         *)
        fun getLine () =
            let
              val line = case TextIO.inputLine sourceStream of NONE => ""
							     | SOME s => s
              val range = (!readChars, !readChars + size line)
            in
              readChars := #2 range;
              if "" = line
              then NONE
              else SOME(removeHeadingAsterisk line, range)
            end

        (**********)
        (* parse of blockTag *)

        fun splitBySpace line = 
            let val substring = SS.extract (line, 0, NONE)
            in SS.splitl (not o Char.isSpace) substring end

        fun parseParamPattern (text, (textBeginPos, _)) =
            let
              val stream = TextIO.openString text
              fun getLine length = case TextIO.inputLine stream of NONE => ""
								 | SOME s => s
              fun onParseError' (message, left, right) =
                  onParseError
                      (message, textBeginPos + left, textBeginPos + right)
              val lexer =
                  ParamPatternParser.makeLexer getLine {error = onParseError'}
              val (patterns, _) =
                  ParamPatternParser.parse
                  (0, lexer, onParseError', ())
            in
              patterns
            end

        fun parseExceptionTag description =
            let
              val (exceptionName, description) = splitBySpace description
              val exceptionID = SS.fields (fn c => c = #".") exceptionName
            in
              ExceptionTag(map SS.string exceptionID, SS.string description)
            end
        fun parseParamsTag (description, range) =
            ParamsTag(parseParamPattern (description, range))
        fun parseParamTag description =
            let val (paramName, description) = splitBySpace description
            in ParamTag (SS.string paramName, SS.string description) end

        (**********)
        (* parse of doc comment *)

        fun triml substring = SS.dropl Char.isSpace substring
        fun isTagBlock line =
            let val dropped = triml line
            in (0 < SS.size dropped) andalso (SS.sub (dropped, 0) = #"@") end
        fun isTagChar c = c = #"@" orelse Char.isAlpha c

        fun parseTagBlocks (firstLine, firstLineRange) tagBlocks =
            let
              val (tag, other) = SS.splitl isTagChar (triml firstLine)
              val other = triml other
              fun untilNextTagBlock descriptions =
                  case getLine () of
                    NONE => (SS.concat(List.rev descriptions), NONE)
                  | SOME (line, range) =>
                    if isTagBlock line
                    then (SS.concat(rev descriptions), SOME(line, range))
                    else untilNextTagBlock (line:: descriptions)
              val (description, nextTag) = untilNextTagBlock [other]
              val descRange =
                  let
                    val descBeginPos =
                        (#1 firstLineRange) +
                        (SS.size firstLine - SS.size other)
                    val descEndPos = descBeginPos + size description
                  in (descBeginPos, descEndPos) end

              val tagBlock = 
                  case SS.string tag of
                    "@author" => SOME(AuthorTag description)
                  | "@contributor" => SOME(ContributorTag description)
                  | "@copyright" => SOME(CopyrightTag description)
                  | "@exception" => SOME(parseExceptionTag description)
                  | "@params" => SOME(parseParamsTag (description, descRange))
                  | "@param" => SOME(parseParamTag description)
                  | "@return" => SOME(ReturnTag description)
                  | "@see" => SOME(SeeTag description)
                  | "@throws" => SOME(parseExceptionTag description)
                  | "@version" => SOME(VersionTag description)
                  | tag =>
                    let val message = "Unknown tag:" ^ tag
                    in
                      onParseError
                          (message, #1 firstLineRange, #2 firstLineRange);
                      NONE
                    end
              val tagBlocks' =
                  case tagBlock of
                    NONE => tagBlocks
                  | SOME tagBlock => tagBlock::tagBlocks
            in
              case nextTag of
                NONE => List.rev tagBlocks'
              | SOME nextTagLine => parseTagBlocks nextTagLine tagBlocks'
            end

        fun untilTagBlock descriptions =
            case getLine () of
              NONE => (SS.concat(List.rev descriptions), [])
            | SOME (line, range) =>
              if isTagBlock line
              then
                (
                  SS.concat(List.rev descriptions),
                  parseTagBlocks (line, range) []
                )
              else untilTagBlock (line::descriptions)

        val (description, tagBlocks) = untilTagBlock []

        val summary =
            let
              fun isSentencePeriod c = c = #"."
              val substring = SS.extract (description, 0, NONE)
              fun findSentencePeriod index =
                  if index = SS.size substring
                  then index
                  else
                    if isSentencePeriod (SS.sub (substring, index))
                       andalso
                       (index = (SS.size substring - 1)
                        orelse
                        Char.isSpace (SS.sub (substring, index + 1)))
                    then Int.min(index + 2, SS.size substring)
                    else findSentencePeriod (index + 1)
              val summaryLength = findSentencePeriod 0
              val summary =
                  if summaryLength = 0
                  then SS.full ""
                  else SS.slice(substring, 0, SOME(summaryLength - 1))
            in SS.string summary end

        (**********)

      in
        ((summary, description, tagBlocks), left, right)
      end

  (****************************************)

  type annotationCTXframe = {fixities : (string * Ast.fixity) list}
  type annotationContext = annotationCTXframe list

  (**
   * indicates whether an ID indicates infix operator.
   * @params (ID, context)
   * @param ID ID
   * @param context context
   * @return true if the ID is included in the context as name of an infix
   *            operator
   *)
  fun isInfixInCTX(name, [] : annotationContext) = false
    | isInfixInCTX(name, {fixities} :: parents) =
      case List.find (fn (opName, fixity) => name = opName) fixities of
        NONE => isInfixInCTX(name, parents)
      | SOME(_, Ast.NONfix) => false
      | SOME(_, _) => true

  (** adds an ID to the context as name of an infix operator
   * @params (fixity, IDs, context)
   * @param fixiy the fixity
   * @param IDs a list of ID
   * @param context the context
   * @return new context extended by the IDs
   *)
  fun addFixityToCTX(fixity, ops, (CTX :: parents) : annotationContext) =
      {fixities = (map (fn name => (name, fixity)) ops) @ (#fixities CTX)} ::
      parents
    | addFixityToCTX(fixity, ops, []) = raise Fail "BUG: empty context"
      
  (**
   *  append two contexts.
   * Operations on the returned context will be performed on the child context
   * and on the parent context if the operation on the child failed.
   * @params (child, parent)
   * @param child child context
   * @param parent parent context
   * @return new context made by pushing the child on the parent.
   *)
  fun pushCTXon (CTXchild, CTXparent : annotationContext) =
      CTXchild @ CTXparent

  (** empty context *)
  val emptyContext = [{fixities = []}]
                     
  (** utility function exported
   * @params (name, context)
   * @param name the ID of infix
   * @param context
   * @return new context extended with the name
   *)
  fun addInfix (name, CTX) = addFixityToCTX(Ast.INfix 0, [name], CTX)

  (**
   * annotates declarations in AST with doc comments.
   * <p>
   *  This function associates a declaration and a doc comment if the doc
   * comment appears at the just before the declaration in the source code.
   * </p>
   * @params parameter context posToLocation (ASTs, docComments)
   * @param parameter General parameter
   * @param context a initial context
   * @param posToLocation a function which converts a pos to a location
   *          (= file name, line number, column number).
   * @param ASTs a list of AST generated by the Parser
   * @param docComments a list of pairs of docComment and its range
   * @return annotated ASTs
   *)
  fun annotate parameter initialCTX posToLocation (asts, docComments) =
      let
        fun getLoc pos = posToLocation pos

        fun findDocComment (regionLeft, regionRight) =
            let
              fun inRegion (_, left, right) =
                  regionLeft < left andalso right < regionRight
            in
              case List.find inRegion docComments of
                NONE => NONE
              | SOME(docComment, _, _) => SOME(docComment)
            end

        fun getBoundNames CTX pat =
            case pat of
              (Ast.WildPat) => []
            | (Ast.VarPat path) =>
              (case path of
                 [id] => [id]
               | _ =>
                 (DGP.warn parameter "LongVID in pattern is not resolved.";
                  []))
            | (Ast.IntPat _) => []
            | (Ast.WordPat _) => []
            | (Ast.StringPat _) => []
            | (Ast.CharPat _) => []
            | (Ast.RecordPat{def, ...}) =>
              List.concat (map (fn(_, pat)=> getBoundNames CTX pat) def)
            | (Ast.ListPat pats) => List.concat (map (getBoundNames CTX) pats)
            | (Ast.TuplePat pats) => List.concat (map (getBoundNames CTX) pats)
            | (Ast.FlatAppPat pats) =>
              (case pats of
                 [leftPat, Ast.VarPat[maybeOpID], rightPat] =>
                 (if isInfixInCTX (maybeOpID, CTX) then [maybeOpID] else []) @
                 (getBoundNames CTX leftPat) @ (getBoundNames CTX rightPat)
               | _ => List.concat (map (getBoundNames CTX) pats))
            | (Ast.ConstraintPat{pattern, ...}) => getBoundNames CTX pattern
            | (Ast.LayeredPat{varPat, expPat}) =>
              (getBoundNames CTX varPat) @ (getBoundNames CTX expPat)
            | (Ast.VectorPat pats) =>
              List.concat (map (getBoundNames CTX) pats)
            | (Ast.MarkPat (pat, _)) => getBoundNames CTX pat
            | (Ast.OrPat pats) =>
              List.concat(map (getBoundNames CTX) pats) (* ?? *)

        fun getFunNameInPats CTX pats =
            case pats of
              [Ast.VarPat[id], Ast.VarPat[maybeOpID], _] =>
              if isInfixInCTX (maybeOpID, CTX) then maybeOpID else id
            | (Ast.VarPat[id])::_ => id
            | _ => (DGP.warn parameter "cannot find functionName."; "?")

        (********************)

        fun annotateSigConst annotate (Ast.NoSig) = AA.NoSig
          | annotateSigConst annotate (Ast.Transparent s) =
            AA.Transparent (annotate s)
          | annotateSigConst annotate (Ast.Opaque s) = AA.Opaque (annotate s)

        fun annotateBindList annotator beginPos binds =
            let
              val (_, annotatedBinds) =
                  foldl
                  (fn ((bind, endPos), (beginPos, annotatedBinds)) =>
                      (endPos, annotatedBinds @ (annotator beginPos bind)))
                  (beginPos, [])
                  binds
            in
              annotatedBinds
            end
        val annotateSpecList = annotateBindList
        val annotateTyList = annotateBindList

        (********************)

        fun annotateStrExp CTX (Ast.VarStr path) = AA.VarStr path
          | annotateStrExp CTX (Ast.BaseStr (decBeginPos, dec)) =
            let val (deltaCTX, annotatedDec) = annotateDec CTX decBeginPos dec
            in AA.BaseStr(annotatedDec) end
          | annotateStrExp CTX (Ast.ConstrainedStr (strExp, sigConst)) =
            AA.ConstrainedStr
            (
              annotateStrExp CTX strExp,
              annotateSigConst annotateSigExp sigConst
            )
          | annotateStrExp CTX (Ast.AppStr(path, strExpAndIsExps)) =
            let
              fun annotate (strExp, isExp) = (annotateStrExp CTX strExp, isExp)
            in AA.AppStr(path, map annotate strExpAndIsExps) end
          | annotateStrExp
            CTX (Ast.LetStr(decBeginPos, dec, strExp)) =
            let val (deltaCTX, annotatedDec) = annotateDec CTX decBeginPos dec
            in
              AA.LetStr
              (annotatedDec, annotateStrExp (pushCTXon(deltaCTX, CTX)) strExp)
            end
          | annotateStrExp CTX (Ast.MarkStr(strExp, _)) =
            annotateStrExp CTX strExp

        and annotateFctExp CTX (Ast.VarFct(path, sigConst)) =
            AA.VarFct(path, annotateSigConst annotateFsigExp sigConst)
          | annotateFctExp CTX (Ast.BaseFct{params, body, constraint}) =
            let fun annotateParam(name, sigExp) = (name, annotateSigExp sigExp)
            in
              AA.BaseFct
              {
                params = map annotateParam params,
                body = annotateStrExp CTX body,
                constraint = annotateSigConst annotateSigExp constraint
              }
            end
          | annotateFctExp CTX (Ast.LetFct(decBeginPos, dec, fctExp)) =
            let val (deltaCTX, annotatedDec) = annotateDec CTX decBeginPos dec
            in
              AA.LetFct
              (annotatedDec, annotateFctExp (pushCTXon(deltaCTX, CTX)) fctExp)
            end
          | annotateFctExp CTX (Ast.AppFct(path, strExpAndIsExps, sigConst)) =
            let
              fun annotate (strExp, isExp) = (annotateStrExp CTX strExp, isExp)
            in
              AA.AppFct
              (
                path,
                map annotate strExpAndIsExps,
                annotateSigConst annotateFsigExp sigConst
              )
            end
          | annotateFctExp CTX (Ast.MarkFct(fctExp, _)) =
            annotateFctExp CTX fctExp

        and annotateWhereSpec (Ast.WhType(qid, tyvars, tyBeginPos, ty)) =
            AA.WhType(qid, tyvars, annotateTy tyBeginPos ty)
          | annotateWhereSpec (Ast.WhStruct(qid1, qid2)) =
            AA.WhStruct(qid1, qid2)

        and annotateSigExp (Ast.VarSig name) = AA.VarSig name
          | annotateSigExp (Ast.AugSig(sigExp, whereSpecs)) =
            AA.AugSig(annotateSigExp sigExp, map annotateWhereSpec whereSpecs)
          | annotateSigExp (Ast.BaseSig (specBeginPos, specs)) =
            AA.BaseSig(annotateSpecList annotateSpec specBeginPos specs)
          | annotateSigExp (Ast.MarkSig (sigExp, _)) = annotateSigExp sigExp

        and annotateFsigExp (Ast.VarFsig name) = AA.VarFsig name
          | annotateFsigExp (Ast.BaseFsig {params, result}) =
            let fun annotateParam(name, sigExp) = (name, annotateSigExp sigExp)
            in
              AA.BaseFsig
              {
                params = map annotateParam params,
                result = annotateSigExp result
              }
            end
          | annotateFsigExp (Ast.MarkFsig (fsigExp, _)) =
            annotateFsigExp fsigExp

        and annotateSpec beginPos (Ast.StrSpec(specs)) =
            let 
              fun annotate beginPos ((name, sigExp, path), (left, _)) =
                  let
                    val docComment = findDocComment (beginPos, left)
                    val loc = getLoc left
                  in
                    [AA.StrSpec
                         (name, loc, annotateSigExp sigExp, path, docComment)]
                  end
            in annotateSpecList annotate beginPos specs end
          | annotateSpec beginPos (Ast.TycSpec(specs, isEqType)) =
            let
              fun annotate
                  beginPos ((name, tyvars, tyBeginPos, tyOpt), (left, _)) =
                  let
                    val docComment = findDocComment (beginPos, left)
                    val annotatedTy = annotateTyOpt tyBeginPos tyOpt
                    val loc = getLoc left
                  in
                    [AA.TycSpec
                    (name, loc, tyvars, annotatedTy, isEqType, docComment)]
                  end
            in annotateSpecList annotate beginPos specs end
          | annotateSpec beginPos (Ast.FctSpec specs) =
            let
              fun annotate beginPos ((name, fsigExp), (left, _)) = 
                  let
                    val docComment = findDocComment (beginPos, left)
                    val loc = getLoc left
                  in
                    [AA.FctSpec
                         (name, loc, annotateFsigExp fsigExp, docComment)]
                  end
            in annotateSpecList annotate beginPos specs end
          | annotateSpec beginPos (Ast.ValSpec specs) =
            let
              fun annotate beginPos ((name, tyBeginPos, ty), (left, _)) =
                  let
                    val docComment = findDocComment (beginPos, left)
                    val loc = getLoc left
                  in
                    [AA.ValSpec
                         (name, loc, annotateTy tyBeginPos ty, docComment)]
                  end
            in annotateSpecList annotate beginPos specs end
          | annotateSpec
            beginPos (Ast.DataSpec{datatycs, withtycsBeginPos, withtycs}) =
            [AA.DataSpec
             {
               datatycs = annotateBindList annotateDB beginPos datatycs,
               withtycs =
               annotateBindList annotateTB withtycsBeginPos withtycs
             }]
          | annotateSpec beginPos (Ast.ExceSpec specs) =
            let
              fun annotate beginPos ((name, tyBeginPos, tyOpt), (left, _)) =
                  let
                    val docComment = findDocComment (beginPos, left)
                    val annotatedTy = annotateTyOpt tyBeginPos tyOpt
                    val loc = getLoc left
                  in [AA.ExceSpec(name, loc, annotatedTy, docComment)] end
            in annotateSpecList annotate beginPos specs end
          | annotateSpec beginPos (Ast.ShareStrSpec paths) =
            [AA.ShareStrSpec paths]
          | annotateSpec beginPos (Ast.ShareTycSpec paths) =
            [AA.ShareTycSpec paths]
          | annotateSpec beginPos (Ast.IncludeSpec sigExp) =
            [AA.IncludeSpec(annotateSigExp sigExp)]

        and annotateDec CTX beginPos (Ast.ValDec(vbs, _)) =
            let
              val annotatedVBs = annotateBindList (annotateVB CTX) beginPos vbs
            in (emptyContext, map AA.ValDec annotatedVBs) end
          | annotateDec CTX beginPos (Ast.ValrecDec(rvbs, _)) =
            let
              val annotatedRVBs =
                  annotateBindList (annotateRVB CTX) beginPos rvbs
            in (emptyContext, map AA.ValDec annotatedRVBs) end
          | annotateDec CTX beginPos (Ast.FunDec (fbs, _)) =
            let
              val annotatedFBs = annotateBindList (annotateFB CTX) beginPos fbs
            in (emptyContext, map AA.FunDec annotatedFBs) end
          | annotateDec CTX beginPos (Ast.TypeDec tbs) =
            let
              val annotatedTBs = annotateBindList annotateTB beginPos tbs
            in (emptyContext, map AA.TypeDec annotatedTBs) end
          | annotateDec
            CTX
            beginPos
            (Ast.DatatypeDec{datatycs, withtycsBeginPos, withtycs}) =
            (
              emptyContext,
              [AA.DatatypeDec
               {
                 datatycs = annotateBindList annotateDB beginPos datatycs,
                 withtycs =
                 annotateBindList annotateTB withtycsBeginPos withtycs
               }]
            )
          | annotateDec
            CTX
            beginPos
            (Ast.AbstypeDec
                 {abstycs, withtycs, withtycsBeginPos, bodyBeginPos, body}) =
            let
              val abstycs' = annotateBindList annotateDB beginPos abstycs
              val withtycs' =
                 annotateBindList annotateTB withtycsBeginPos withtycs
              val (_, body') = annotateDec CTX bodyBeginPos body
            in
              (
                emptyContext,
                [AA.AbstypeDec
                    {
                      datatycs = abstycs',
                      withtycs =  withtycs',
                      body = body'
                    }]
              )
            end
          | annotateDec CTX beginPos (Ast.ExceptionDec ebs) =
            let val annotatedEBs = annotateBindList annotateEB beginPos ebs
            in (emptyContext, map AA.ExceptionDec annotatedEBs) end
          | annotateDec CTX beginPos (Ast.StrDec strbs) =
            let
              val annotatedSTRBs =
                  annotateBindList (annotateSTRB CTX) beginPos strbs
            in (emptyContext, map AA.StrDec annotatedSTRBs) end
          | annotateDec CTX beginPos (Ast.AbsDec strbs) =
            let
              val annotatedSTRBs =
                  annotateBindList (annotateSTRB CTX) beginPos strbs
            in (emptyContext, map AA.StrDec annotatedSTRBs) end
          | annotateDec CTX beginPos (Ast.FctDec fctbs) =
            let
              val annotatedFCTBs =
                  annotateBindList (annotateFCTB CTX) beginPos fctbs
            in (emptyContext, map AA.FctDec annotatedFCTBs) end
          | annotateDec CTX beginPos (Ast.SigDec sigbs) =
            let
              val annotatedSIGBs = annotateBindList annotateSIGB beginPos sigbs
            in (emptyContext, map AA.SigDec annotatedSIGBs) end
          | annotateDec CTX beginPos (Ast.FsigDec fsigbs) =
            let
              val annotatedFSIGBs =
                  annotateBindList annotateFSIGB beginPos fsigbs
            in (emptyContext, map AA.FsigDec annotatedFSIGBs) end
          | annotateDec
            CTX
            _
            (Ast.LocalDec (localBegin, localDec, globalBegin, globalDec)) =
            let
              val (deltaCTX, localDecs') = annotateDec CTX localBegin localDec
              val (deltaCTX', globalDecs') =
                  annotateDec (pushCTXon(deltaCTX, CTX)) globalBegin globalDec
            in
              (deltaCTX', [AA.LocalDec(localDecs', globalDecs')])
            end
          | annotateDec CTX beginPos (Ast.SeqDec decs) =
            let
              val (deltaCTX, beginPos, annotatedDecs) =
                  foldl
                  (fn ((dec, endPos), (deltaCTX, beginPos, annotatedDecs)) =>
                      let
                        val (deltaCTX', annotatedDecs') =
                            annotateDec (pushCTXon(deltaCTX, CTX)) beginPos dec
                      in
                        (
                          pushCTXon(deltaCTX', deltaCTX),
                          endPos,
                          annotatedDecs @ annotatedDecs'
                        )
                      end)
                  (emptyContext, beginPos, []) decs
            in (deltaCTX, annotatedDecs) end
          | annotateDec CTX _ (Ast.OpenDec paths) =
            (emptyContext, map AA.OpenDec paths)
          | annotateDec CTX _ (Ast.OvldDec _) = (emptyContext, [])
          | annotateDec CTX _ (Ast.FixDec{fixity, ops}) =
            (addFixityToCTX (fixity, ops, CTX), [])
          | annotateDec CTX _ (Ast.UseDec _) = (emptyContext, [])
          | annotateDec CTX beginPos (Ast.MarkDec(dec, _)) =
            annotateDec CTX beginPos dec

        and annotateVB CTX beginPos (Ast.Vb({pat, ...}, (left, _))) =
            let
              (* ToDo : more precise location *)
              val boundNames = getBoundNames CTX pat
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in map (fn name => (name, loc, docComment)) boundNames end

        and annotateRVB CTX beginPos (Ast.Rvb({var, ...}, (left, _))) =
            let
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in [(var, loc, docComment)] end

        and annotateFB CTX beginPos (Ast.Fb(clause::_, _)) =
            annotateClause CTX beginPos clause
          | annotateFB CTX beginPos (Ast.Fb([], _)) =
            []
            (* ToDO : not warn ? *)
(*
            (raise ParseError "empty function binds.")
*)
          | annotateFB CTX beginPos (Ast.MarkFb(fb, _)) =
            annotateFB CTX beginPos fb

        and annotateClause CTX beginPos (Ast.Clause({pats, ...}, (left, _))) =
            let
              val funName = getFunNameInPats CTX pats
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in [(funName, loc, docComment)] end

        and annotateTB
            beginPos (Ast.Tb({tyc, defBeginPos, def, tyvars}, (left, _))) =
            let
              val annotatedTy = annotateTy defBeginPos def
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in [AA.Tb(tyc, loc, tyvars, SOME annotatedTy, docComment)] end

        and annotateDB
            beginPos
            (Ast.Db({tyc, tyvars, rhsBeginPos, rhs, ...}, (left, _))) =
            let
              val annotatedRhs = annotateDBRHS rhsBeginPos rhs
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in
              [AA.Db
               (
                 {tyc = tyc, loc = loc, tyvars = tyvars, rhs = annotatedRhs},
                 docComment
               )]
            end

        and annotateDBRHS beginPos (Ast.Constrs(constrs)) =
            let
              val annotatedConstrs =
                  annotateBindList annotateConstr beginPos constrs
            in AA.Constrs(annotatedConstrs) end
          | annotateDBRHS beginPos (Ast.Repl path) = AA.Repl path

        and annotateConstr beginPos ((valCon, tyBeginPos, tyOpt), (left, _)) =
            let
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in [(valCon, loc, annotateTyOpt tyBeginPos tyOpt, docComment)] end

        and annotateEB
            beginPos (Ast.EbGen({exn, etypeBeginPos, etype}, (left, _))) =
            let
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in
              [AA.EbGen
                   (exn, loc, annotateTyOpt etypeBeginPos etype, docComment)]
            end
          | annotateEB beginPos (Ast.EbDef({exn, edef}, (left, _))) =
            let
              val docComment = findDocComment (beginPos, left)
              val loc = getLoc left
            in [AA.EbDef(exn, loc, edef, docComment)] end

        and annotateSTRB
            CTX
            beginPos
            (Ast.Strb({name, def, constraint}, (left, _))) =
            let
              val docComment = findDocComment (beginPos, left)
            in
              [(
                name,
                getLoc left,
                annotateStrExp CTX def,
                annotateSigConst annotateSigExp constraint,
                docComment
              )]
            end

        and annotateFCTB
            CTX beginPos (Ast.Fctb({name, def}, (left, _))) =
            let val docComment = findDocComment (beginPos, left)
            in [(name, getLoc left, annotateFctExp CTX def, docComment)] end

        and annotateSIGB beginPos (Ast.Sigb({name, def}, (left, _))) =
            let val docComment = findDocComment (beginPos, left)
            in [(name, getLoc left, annotateSigExp def, docComment)] end

        and annotateFSIGB beginPos (Ast.Fsigb({name, def}, (left, _))) =
            let val docComment = findDocComment (beginPos, left)
            in [(name, getLoc left, annotateFsigExp def, docComment)] end

        and annotateTy beginPos (Ast.VarTy(tyvar)) = AA.VarTy(tyvar)
          | annotateTy beginPos (Ast.ConTy(qid, tysBeginPos, tys)) =
            let
              val tysBeginPos' =
                  if tysBeginPos = 0 then beginPos else tysBeginPos
              fun annotate beginPos ty = [annotateTy beginPos ty]
            in AA.ConTy(qid, annotateTyList annotate tysBeginPos' tys) end
          | annotateTy beginPos (Ast.RecordTy(tyrowsBeginPos, tyrows)) =
            AA.RecordTy(annotateTyList annotateTyRow tyrowsBeginPos tyrows)
          | annotateTy beginPos (Ast.TupleTy(elems)) =
            let fun annotate beginPos ty = [annotateTy beginPos ty]
            in AA.TupleTy(annotateTyList annotate beginPos elems) end
          | annotateTy beginPos (Ast.EnclosedTy(innerBeginPos, ty)) =
            annotateTy innerBeginPos ty
          | annotateTy beginPos (Ast.MarkTy(ty, (left, _))) =
            let
              val docCommentOpt = findDocComment (beginPos, left)
              val ty' = annotateTy left ty
            in
              case docCommentOpt of
               SOME docComment => AA.CommentedTy(docComment, ty')
             | NONE => ty'
            end

        and annotateTyRow
            beginPos (Ast.TyRow((label, tyBeginPos, ty), (left, _))) =
            let val docComment = findDocComment (beginPos, left)
            in [(label, annotateTy tyBeginPos ty, docComment)] end

        and annotateTyOpt beginPos NONE = NONE
          | annotateTyOpt beginPos (SOME ty) = SOME (annotateTy beginPos ty)

        val (_, _, annotatedAsts) =
            foldl
            (fn ((ast, endPos), (CTX, beginPos, annotatedAsts)) =>
                let
                  val (deltaCTX, annotatedAsts') = annotateDec CTX beginPos ast
                  val CTX' = pushCTXon(deltaCTX, CTX)
                in (CTX', endPos, annotatedAsts' :: annotatedAsts) end)
            (initialCTX, 0, [])
            asts
      in
        List.concat(List.rev annotatedAsts)
      end

  (****************************************)

  (** context *)
  type context = annotationContext

  (**
   * parses a file.
   * @params parameter context fileName
   * @param parameter general parameter
   * @param context a initial context
   * @param fileName name of the source file
   * @return annotated ASTs
   *)
  fun parseFile parameter CTX fileName = 
      let
        val _ = DGP.onProgress parameter ("Parsing code: " ^ fileName)
        val (asts, docCommentTexts, parserOperations) =
            parseML parameter fileName

        val _ = DGP.onProgress parameter ("Parsing docComments: " ^ fileName)
        val parsedDocComments =
            map (parseDocComment parameter parserOperations) docCommentTexts

        val _ = DGP.onProgress parameter ("Annotating: " ^ fileName)
        fun posToLocation pos =
            let val (line, column) = #posToLocation parserOperations pos
            in {fileName = fileName, line = line, column = column} end
        val annotatedAsts =
            annotate parameter CTX posToLocation (asts, parsedDocComments)
      in
        AA.CompileUnit(fileName, annotatedAsts)
      end

  (***************************************************************************)

end