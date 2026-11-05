In gmr

1) Rem in Cargo.toml:
```toml
[package.metadata.leptos]
# bin-target-triple = "x86_64-pc-windows-msvc"
```

2) Then:
```
docker build -t cx58 .
docker tag cx58 alsh/cx58
docker push alsh/cx58
or
docker build --progress=plain --no-cache -t cx58 .  1> build1.log 2>build2.log
docker build --progress=plain -t cx58 .  1> build1.log 2>build2.log

```

3) Run
```
docker run -p 3000:3000 --env-file remote.env cx58:latest
```

4) Unrem in Cargo.toml:
```toml
[package.metadata.leptos]
bin-target-triple = "x86_64-pc-windows-msvc"
```
