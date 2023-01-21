<div align="center">
  <h1>Turborepo Server</h1>
  <p>
    <b>
      A remote cache backend for
      <a href="https://turbo.build/repo" target="_blank">Turborepo</a> written in Rust.
    </b>
  </p>
  <sub>
    Built on top of
    <a href="https://github.com/hyperium/hyper" target="_blank">Hyper</a>
  </sub>
</div>

## Abstract
As the website states "[Turborepo](https://turbo.build/repo) is a high-performance
build system for JavaScript and TypeScript codebases".

Out-of-the-box, the Turborepo allows to cache transpilation and build artifact on
locally. Turborepo also allow to cache the ouput directly with [Vercel](https://vercel.com/dashboard),
and even pass a custom URL for on-premises caching servers.

Looking around, I found a handful of Turborepo compatible servers, but none writte
in Rust. So, I decided to build a barebone Turborepo remote caching server.

This project is also an opportunity to practice and improve my Rust.

## Run the server

The server can run through Cargo.

```sh
cargo run --release serve \
  --api-port 3000 \
  --bucket "bucket-name" \
  --token "aaa"
```

By default, the server stores cache in on the local filesystem.
To enable AWS S3 caching, the `--storage` flag must be set to `aws` as follow.

```sh
cargo run --release serve \
  --api-port 3000 \
  --bucket "bucket-name" \
  --token "aaa" \
  --storage aws
```

## Known issues

- [ ] At the moment, `--token` is required, but is not actually used.
- [ ] AWS upload is not optimised and takes ages (especially for large files).

## Inspiration

- [Topico Turborepo remote cache](https://github.com/Tapico/tapico-turborepo-remote-cache) in Go
- [Turborepo remote cache](https://github.com/ducktors/turborepo-remote-cache) in TypeScript

## License

_Turborepo Server_ is distributed under the terms of the MIT license.

See [LICENSE](LICENSE) for details.