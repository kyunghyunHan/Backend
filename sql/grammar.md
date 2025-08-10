# PostgreSQL Grammar

## CREATE ROLE
- 새로운 역할(유저) 생성
```sql
CREATE ROLE admin WITH LOGIN PASSWORD 'qwer1234';
```

## CREATE DATABASE
- 새로운 데이터베이스 생성
```sql
CREATE DATABASE mydb OWNER admin;
```

## CREATE TABLE
- 테이블 생성
```sql
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50),
    hire_date DATE,
    salary NUMERIC(10, 2)
);
```

## INSERT
- 데이터 삽입
```sql
INSERT INTO employees (name, hire_date, salary)
VALUES ('John', '2025-01-10', 3500.00);
```

## UPDATE
- 데이터 수정
```sql
UPDATE employees
SET salary = salary * 1.1
WHERE id = 1;
```

## DELETE
- 데이터 삭제
```sql
DELETE FROM employees WHERE id = 1;
```

## COALESCE
- 첫 번째로 NULL이 아닌 값을 반환
```sql
SELECT COALESCE(NULL, 'Hello', 'World');
```

## NULLIF
- 두 값이 같으면 NULL, 다르면 첫 번째 값 반환
```sql
SELECT NULLIF(10, 10);  -- 결과: NULL
SELECT NULLIF(10, 20);  -- 결과: 10
```

## ROUND
- 숫자를 반올림
```sql
SELECT ROUND(3.14159, 2); -- 결과: 3.14
```

## CAST (::)
- 데이터 타입 변환
```sql
SELECT '123'::int;
SELECT CAST('123' AS int);
```

## COUNT DISTINCT (다중 컬럼)
- 여러 컬럼의 유니크 조합 개수
```sql
SELECT COUNT(DISTINCT (col1, col2)) FROM mytable;
```

## DATE_TRUNC
- 날짜를 특정 단위로 절삭
```sql
SELECT DATE_TRUNC('month', NOW()); -- 이번 달 1일 00:00:00
```

## EXTRACT
- 날짜/시간에서 특정 부분 추출
```sql
SELECT EXTRACT(YEAR FROM NOW());
SELECT EXTRACT(MONTH FROM NOW());
```

## GROUP BY
- 그룹별 집계
```sql
SELECT department, COUNT(*) 
FROM employees 
GROUP BY department;
```

## HAVING
- 그룹 조건
```sql
SELECT department, COUNT(*) 
FROM employees 
GROUP BY department
HAVING COUNT(*) > 5;
```

## CASE
- 조건 분기
```sql
SELECT 
    CASE 
        WHEN score >= 90 THEN 'A'
        WHEN score >= 80 THEN 'B'
        ELSE 'C'
    END AS grade
FROM students;
```

## WINDOW FUNCTION (ROW_NUMBER)
- 순위 매기기
```sql
SELECT name, salary,
       ROW_NUMBER() OVER (ORDER BY salary DESC) AS rank
FROM employees;
```

## WINDOW FUNCTION (RANK & DENSE_RANK)
```sql
SELECT name, salary,
       RANK() OVER (ORDER BY salary DESC) AS rnk,
       DENSE_RANK() OVER (ORDER BY salary DESC) AS dense_rnk
FROM employees;
```

## JOIN
- INNER JOIN
```sql
SELECT e.name, d.department_name
FROM employees e
JOIN departments d ON e.department_id = d.id;
```

- LEFT JOIN
```sql
SELECT e.name, d.department_name
FROM employees e
LEFT JOIN departments d ON e.department_id = d.id;
```

## SUBQUERY
- 서브쿼리 예시
```sql
SELECT name, salary
FROM employees
WHERE salary > (SELECT AVG(salary) FROM employees);
```

## LIMIT / OFFSET
- 페이징 처리
```sql
SELECT * FROM employees
ORDER BY id
LIMIT 10 OFFSET 20;
```
