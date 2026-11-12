use quote::ToTokens;
use serde::Serialize;
use syn::visit::{self, Visit};
use syn::{
    Expr, ExprCall, ExprMethodCall, FnArg, ImplItemFn, ItemFn, ItemImpl, ItemTrait, ReturnType,
    Signature, TraitItemFn, Type,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionKind {
    Standalone,
    Associated,
    Method,
}

#[derive(Debug, Clone, Serialize)]
pub struct Param {
    pub name: String, // "self" for receivers
    pub ty: String,   // e.g. "&mut self", "Vec<String>"
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub id: usize,
    pub kind: FunctionKind,
    pub name: String,
    pub owner: Option<String>, // None for standalone fns
    pub params: Vec<Param>,
    pub return_type: Option<String>, // None (null in JSON) when it returns ()
    pub calls: Vec<usize>,           // ids of user-defined fns this fn calls
}

#[derive(Serialize)]
pub struct Hierarchy<'a> {
    pub roots: Vec<usize>, // fns that no other user fn calls
    pub functions: &'a [FunctionInfo],
}

enum RawCall {
    Path { owner: Option<String>, name: String }, // foo() or Type::foo()
    Method { name: String },                      // x.foo()
}

#[derive(Default)]
pub struct UserDefined {
    pub fns: Vec<FunctionInfo>, // the main Vec
    owner_stack: Vec<String>,
    fn_stack: Vec<usize>,
    raw_calls: Vec<(usize, RawCall)>,
}

impl UserDefined {
    pub fn new(ast: &syn::File) -> Self {
        let mut v = Self::default();
        v.visit_file(ast);
        v.resolve_calls();
        v
    }

    pub fn is_user_defined(&self, owner: Option<&str>, name: &str) -> bool {
        self.fns
            .iter()
            .any(|f| f.name == name && f.owner.as_deref() == owner)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.hierarchy())
    }

    pub fn hierarchy(&self) -> Hierarchy<'_> {
        let mut called = vec![false; self.fns.len()];
        for f in &self.fns {
            for &c in &f.calls {
                if c != f.id {
                    called[c] = true;
                }
            }
        }
        let roots = (0..self.fns.len()).filter(|&i| !called[i]).collect();
        Hierarchy { roots, functions: &self.fns }
    }

    fn push_fn(&mut self, kind: FunctionKind, sig: &Signature, owner: Option<String>) -> usize {
        let id = self.fns.len();
        let (params, return_type) = signature_parts(sig);
        self.fns.push(FunctionInfo {
            id,
            kind,
            name: sig.ident.to_string(),
            owner,
            params,
            return_type,
            calls: Vec::new(),
        });
        id
    }

    // Match every recorded call against the user-defined fns.
    // Anything that doesn't match (String::new, .len(), ...) is a built-in and is dropped.
    fn resolve_calls(&mut self) {
        for (caller, raw) in std::mem::take(&mut self.raw_calls) {
            let targets: Vec<usize> = self
                .fns
                .iter()
                .filter(|f| match &raw {
                    RawCall::Path { owner, name } => f.name == *name && f.owner == *owner,
                    // method calls can't be type-resolved, so match by name (heuristic)
                    RawCall::Method { name } => f.kind == FunctionKind::Method && f.name == *name,
                })
                .map(|f| f.id)
                .collect();
            for t in targets {
                if !self.fns[caller].calls.contains(&t) {
                    self.fns[caller].calls.push(t);
                }
            }
        }
    }
}

fn type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Type::Reference(r) => type_name(&r.elem),
        _ => None,
    }
}

fn kind_of(sig: &Signature) -> FunctionKind {
    if sig.receiver().is_some() {
        FunctionKind::Method
    } else {
        FunctionKind::Associated
    }
}

// Token streams print with extra spaces ("Vec < String >"), so tidy them up.
fn clean(s: String) -> String {
    s.replace(" :: ", "::")
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" <", "<")
        .replace(" >", ">")
        .replace("& ", "&")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" ,", ",")
}

fn signature_parts(sig: &Signature) -> (Vec<Param>, Option<String>) {
    let params = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => {
                let text = clean(r.to_token_stream().to_string()); // "&mut self"
                Param { name: "self".to_string(), ty: text }
            }
            FnArg::Typed(p) => Param {
                name: clean(p.pat.to_token_stream().to_string()),
                ty: clean(p.ty.to_token_stream().to_string()),
            },
        })
        .collect();

    let return_type = match &sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(clean(ty.to_token_stream().to_string())),
    };

    (params, return_type)
}

impl<'ast> Visit<'ast> for UserDefined {
    fn visit_item_fn(&mut self, f: &'ast ItemFn) {
        let id = self.push_fn(FunctionKind::Standalone, &f.sig, None);
        self.fn_stack.push(id);
        visit::visit_item_fn(self, f);
        self.fn_stack.pop();
    }

    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        self.owner_stack.push(type_name(&i.self_ty).unwrap_or_default());
        visit::visit_item_impl(self, i);
        self.owner_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, f: &'ast ImplItemFn) {
        let owner = self.owner_stack.last().cloned();
        let id = self.push_fn(kind_of(&f.sig), &f.sig, owner);
        self.fn_stack.push(id);
        visit::visit_impl_item_fn(self, f);
        self.fn_stack.pop();
    }

    fn visit_item_trait(&mut self, t: &'ast ItemTrait) {
        self.owner_stack.push(t.ident.to_string());
        visit::visit_item_trait(self, t);
        self.owner_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, f: &'ast TraitItemFn) {
        let owner = self.owner_stack.last().cloned();
        let id = self.push_fn(kind_of(&f.sig), &f.sig, owner);
        self.fn_stack.push(id);
        visit::visit_trait_item_fn(self, f); // walks the default body, if any
        self.fn_stack.pop();
    }

    fn visit_expr_call(&mut self, c: &'ast ExprCall) {
        let caller = self.fn_stack.last().copied();
        if let (Some(caller), Expr::Path(p)) = (caller, &*c.func) {
            let segs: Vec<String> = p.path.segments.iter().map(|s| s.ident.to_string()).collect();
            let raw = match segs.as_slice() {
                [name] => Some(RawCall::Path { owner: None, name: name.clone() }),
                [.., owner, name] => {
                    let owner = if owner == "Self" {
                        self.owner_stack.last().cloned().unwrap_or_default()
                    } else {
                        owner.clone()
                    };
                    Some(RawCall::Path { owner: Some(owner), name: name.clone() })
                }
                [] => None,
            };
            if let Some(raw) = raw {
                self.raw_calls.push((caller, raw));
            }
        }
        visit::visit_expr_call(self, c); // calls nested in arguments
    }

    fn visit_expr_method_call(&mut self, m: &'ast ExprMethodCall) {
        if let Some(&caller) = self.fn_stack.last() {
            self.raw_calls.push((caller, RawCall::Method { name: m.method.to_string() }));
        }
        visit::visit_expr_method_call(self, m);
    }
}