# Rust 설치 및 생성 가이드

1. `c++` 빌드 도구 설치
Rust는 컴파일 시 C++ 빌드 도구가 필요합니다.
Visual Studio 2022를 통해서 `C++를 사용한 데스크톱 개발`이 체크된 상태로 설치되어 있으면 상관없으나, 
만약 설치되어 있지 않다면 `Visual Studio Installer`를 수정으로 선택하여 `C++를 사용한 데스크톱 개발`을 체크 후 설치해야 합니다.

2. Rust 설치 도구 다운로드

공식 설치 페이지에서 운영체제 환경에 맞는 설치 파일을 다운로드하면 됩니다.
<https://www.rust-lang.org/tools/install>

설치 완료 후 터미널에서

```rust
rustc --version

cargo --version
```

명령어를 통해 잘 설치가 된지 확인합니다.

3. 새로운 프로젝트 생성하기
Rust는 기본적으로 `cargo` 명령어를 통해서 프로젝트를 설치합니다.

프로젝트를 생성을 원하는 디렉토리로 이동 후 
```
cargo new `project_name`    // `project_name`에 원하는 프로젝트 명을 입력한다.
```
위 명령어를 통해서 생성할 수 있습니다.

생성된 프로젝트 디렉토리로 이동하여 
`cargo.toml`과 같은 디렉토리에 내에서 
```rust
cargo run 
```
명령어를 입력하면 빌드 후 실행을 시켜준다.

만약 빌드만 하기를 원한다면

```rust
cargo build // 기본으로 debug 빌드
cargo build --release // release 빌드 
```

# tarui 설치 가이드

1. <https://v2.tauri.app/> <- 공식 사이트를 들어간다.
2. `Get started`를 누른다.
3. 왼쪽 패널에서 `Quick Start` - `Create a Project`를 누른다.

여러 패키지 관리자를 통해서 프로젝트를 만들 수 있다.
여기서는 `npm`을 통해 설치해 보겠다.

`npm`을 먼저 설치해야 한다.
<https://nodejs.org/ko/> <- 여기서 `Node.js`를 설치하면 된다.
그러면 자동으로 `npm`도 설치된다.

설치 이 후 `Get started`에 나온 데로 따라 하면 프로젝트를 생성할 수 있다.

```npm
npm tauri run dev // 개발 모드로 실행 - 핫 리로드 기능 지원
npm tauri run build // release로 빌드가 됨.
                    // ./target/release 경로 아래에 `exe` 파일이 생성된다.
```

파일 구조
- `tauri.conf.json`: Tauri 애플리케이션의 설정 파일입니다. 창 크기, 메뉴, 권한 등을 설정합니다.
- `src-tauri`: Rust 코드가 위치하는 폴더입니다.
- `src`: 프론트엔드 코드가 위치하는 폴더입니다.


# 개발 도구(IDE) 및 익스텐션 추천
IDE
- vscode 
- NeoVim

익스텐션
- rust-analyzer: Rust 개발의 핵심 확장 프로그램임. 자동 완성, 오류 진단, 정의로 이동, 이름 바꾸기 등 필수적인 기능을 제공해 줌.
- Dependi: 의존성 관리 시 최신 모듈인지 확인할 수 있음
- Better TOML: `Cargo.toml`파일의 구문 강조 기능을 향상시켜 가독성을 높여줌



