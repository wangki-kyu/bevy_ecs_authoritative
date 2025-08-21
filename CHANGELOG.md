. Minor | - Fix | + Addition | ^ improvement | ! Change | * Refactor | @ Version

# 0.1.1-dev  
`+` Addition: test clinet code 작성 중.  
    WebSocket 통신 및 리소스, 이벤트 구현 중.  

## 2025.08.16  
`+` Addition: client에서 키보드 입력 이벤트 발생 시 WebSocket으로 연결된 서버로 보낼 수 있도록 기능 구현 중...  
`-` Fix: 서버에서 `test.rs`에서 테스트 클라이언트로 방향키를 눌렀을 때 받는 코드 작성 완료.  

## 2025.08.19
`+` Addition: `client`에 `Ball` Component를 가진 Entity 생성  
`+` Addition: `server`에서 클라이언트 엔티티 이동 요청을 받았을 때, 로직을 처리하기 위한 `ClientMoveEvent` event 및 시스템 추가 

## 2025.08.21  
`-` Fix: `client` Entity를 생성 시 `uuid`를 가질 수 있도록 `Client` Component가 uuid를 가지도록 튜플 구조체로 처리하였음.  
`+` Addition: sink의 경우도 각각 연결된 클라이언트를 통해 만들어진 클라이언트 엔티티가 개별적으로 websocket sink를 가져야하므로 Component를 추가하였음.  

# 0.1.0
`@` Version: first commit
