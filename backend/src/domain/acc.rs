//! Модуль с алгоритмами для вычисления долгов в комнате.
//!
//! ## Общая идея
//!
//! На основе данных о пользователях в комнате, а именно:
//! 1. Порциях
//! 2. Переводах
//! 3. Тратах
//!
//! Подсчитывается статистика о пользователе за определенный период.
//!
//! На основе этой статистики (а также учета удаленных пользователей)
//! можно посчитать вклад каждого участника - это сумма того, что
//! он вложил в комнату минус то, что он получил от комнаты.
//!
//! Данный вклад в комнате должен быть примерно одинаковым для
//! каждого пользователя.
//! Для этого можно посчитать среднее значение и с помощью
//! переводов привести значения каждого участника к среднему.
//!
//! Таким образом:
//! - `calc_stats` считает статистику по пользователям,
//! - `accounting` вычисляет оптимальное число переводов для усреднения вклада.
//!
//!
//! ## Система порций
//!
//! Иногда участник комнаты не ест в комнате, причем, возможно, достаточно долго
//! (уехал на неделю). Чтобы он за это время не платил за еду, но платил за какие-то
//! более глобальные покупки (покупка техники, ремонт и тд), существует система порций.
//!
//! На каждый день указывается число порций каждого участника.
//!
//! Тогда
//!
//! $$price = \frac{\sum food_purchase}{\sum portion_cnt}$$
//!
//! Где суммы берутся за последние 30 дней.
//!
//! Тогда получим `food_spent` - кто сколько наел еды. Это произведение числа
//! порций учатника в этот день на стоимость порции в этот день.
//! Каждый день стоимость порции может быть разной.
use chrono::{Duration, NaiveDate};
use std::{
    cmp::min,
    collections::BTreeSet,
    ops::{AddAssign, SubAssign},
};
use uuid::Uuid;

/// Количество дней, за время которых вычисляется стоимость порции.
const PRICE_PERIOD: Duration = Duration::days(30);

/// Статистика по пользователю.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberStat {
    pub id: Uuid,

    pub portions: i64,
    pub food_purchases: i64,
    pub not_food_purchases: i64,
    pub transactions: i64,
    pub food_spent: i64,
}

/// Дельта. Говорит о том, насколько определенный пользователь
/// имеет вклад больше/меньше среднего.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Delta {
    val: i64,

    id: Uuid,
}

/// Результат алгоритма по минимизации транзакций.
///
/// Показывает необходимость перевода от человека
/// к человеку для усреднения баланса.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeedTransaction {
    pub member_from: Uuid,
    pub member_to: Uuid,
    pub money: i64,
}

/// Описание одной порции.
#[derive(Clone, Copy)]
pub struct Portion {
    pub member: Uuid,

    pub date: NaiveDate,
    pub cnt: i64,
}

/// Описание одной транзакции.
#[derive(Clone, Copy)]
pub struct Transaction {
    pub member_to: Uuid,
    pub member_from: Uuid,

    pub money: i64,
}

/// Описание одной траты.
#[derive(Clone, Copy)]
pub struct Purchase {
    pub member: Uuid,

    pub money: i64,
    pub is_food: bool,

    pub date: NaiveDate,
}

/// Алгоритм, вычисляющий минимальное число переводов
/// для баланса вкладов.
pub fn accounting(members: &[MemberStat]) -> Vec<NeedTransaction> {
    // Для пустого числа пользователей ответ пустой.
    if members.is_empty() {
        return Vec::new();
    }

    // Считаем сумму вкладов, средний вклад, остаток.
    // Остаток необходим, так как ровно rem пользователей
    // должны иметь среднее на 1 больше, чтобы получить сумму total
    // total = cnt * mean + rem = rem * (mean + 1) + (cnt - rem) * mean
    let total: i64 = members.iter().map(|x| x.total()).sum();
    let cnt = members.len() as i64;
    let mean = total.div_euclid(cnt);
    let rem = total.rem_euclid(cnt);

    // Алгоритм работает следующим образом:
    // - Итеративно берем учатника с минимальным и максимальным вкладом (<0 и >0).
    // - Делаем между ними макс. возможный перевод.
    // - У одного из участников delta = 0 - вклад средний.
    let mut s: BTreeSet<_> = members
        .iter()
        .enumerate()
        .map(|(i, x)| x.delta(mean, i as i64, rem))
        .filter(|x| !x.is_zero())
        .collect();

    let mut res = Vec::with_capacity(members.len() - 1);
    while s.len() >= 2 {
        let mut a = s.pop_first().unwrap();
        let mut b = s.pop_last().unwrap();
        let money = a.send_money(b);

        res.push(NeedTransaction {
            member_from: a.id,
            member_to: b.id,
            money,
        });

        a += money;
        b -= money;
        if !a.is_zero() {
            s.insert(a);
        }
        if !b.is_zero() {
            s.insert(b);
        }
    }

    res
}

/// Алгоритм для вычисления статистики.
///
/// Подаются данные за определенный период,
/// на основе чего считается статистика.
pub fn calc_stats(
    mut members: Vec<Uuid>,
    mut portions: Vec<Portion>,
    trs: &[Transaction],
    mut prs: Vec<Purchase>,
) -> Vec<MemberStat> {
    if members.is_empty() {
        return Vec::new();
    }

    members.sort();
    portions.sort_by_key(|x| x.date);
    prs.sort_by_key(|x| x.date);

    // Индексация в массиве по UUID.
    let idx = |id: Uuid| members.binary_search(&id).unwrap();

    let mut res: Vec<_> = members.iter().map(|x| MemberStat::new(*x)).collect();

    for t in trs {
        res[idx(t.member_from)].transactions += t.money;
        res[idx(t.member_to)].transactions -= t.money;
    }
    for p in &prs {
        if p.is_food {
            res[idx(p.member)].food_purchases += p.money;
        } else {
            res[idx(p.member)].not_food_purchases += p.money;
        }
    }
    for p in &portions {
        res[idx(p.member)].portions += p.cnt;
    }

    let mut port_i = 0;
    let mut port_j = 0;

    let mut prs_i = 0;
    let mut prs_j = 0;

    let mut portions_cnt = 0;
    let mut food_purchases = 0;

    while port_j < portions.len() {
        let date = portions[port_j].date;
        let start_date_index = port_j;

        while port_j < portions.len() && portions[port_j].date <= date {
            portions_cnt += portions[port_j].cnt;
            port_j += 1;
        }
        while port_i < port_j && portions[port_i].date <= date - PRICE_PERIOD {
            portions_cnt -= portions[port_i].cnt;
            port_i += 1;
        }

        while prs_j < prs.len() && prs[prs_j].date <= date {
            if prs[prs_j].is_food {
                food_purchases += prs[prs_j].money;
            }
            prs_j += 1;
        }
        while prs_i < prs_j && prs[prs_i].date <= date - PRICE_PERIOD {
            if prs[prs_i].is_food {
                food_purchases -= prs[prs_i].money;
            }
            prs_i += 1;
        }

        if portions_cnt != 0 {
            for p in &portions[start_date_index..port_j] {
                res[idx(p.member)].food_spent +=
                    (food_purchases * p.cnt + portions_cnt / 2) / portions_cnt;
            }
        }
    }

    res
}

impl MemberStat {
    /// Создание пустой статистики по `id` пользователя.
    fn new(id: Uuid) -> Self {
        Self {
            id,
            portions: 0,
            food_purchases: 0,
            not_food_purchases: 0,
            transactions: 0,
            food_spent: 0,
        }
    }

    /// Суммарный вклад пользователя.
    pub fn total(&self) -> i64 {
        self.food_purchases + self.not_food_purchases + self.transactions - self.food_spent
    }

    /// Получение дельты.
    ///
    /// Если idx < rem, то за среднее считается mean + 1,
    /// иначе просто mean.
    fn delta(&self, mut mean: i64, idx: i64, rem: i64) -> Delta {
        if idx < rem {
            mean += 1;
        }

        Delta {
            val: self.total() - mean,
            id: self.id,
        }
    }
}

impl Delta {
    /// Дельта 0 - вклад уже средний.
    fn is_zero(&self) -> bool {
        self.val == 0
    }

    /// Максимальный размер перевода от
    /// текущего пользователя к другому.
    ///
    /// Увеличивает вклад текущего и уменьшает вклад другого.
    fn send_money(&self, to: Delta) -> i64 {
        min(-self.val, to.val).max(0)
    }
}

impl AddAssign<i64> for Delta {
    fn add_assign(&mut self, rhs: i64) {
        self.val += rhs;
    }
}

impl SubAssign<i64> for Delta {
    fn sub_assign(&mut self, rhs: i64) {
        self.val -= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn create_portion(member: Uuid, date: NaiveDate, cnt: i64) -> Portion {
        Portion { member, date, cnt }
    }

    fn create_transaction(member_from: Uuid, member_to: Uuid, money: i64) -> Transaction {
        Transaction {
            member_from,
            member_to,
            money,
        }
    }

    fn create_purchase(member: Uuid, money: i64, is_food: bool, date: NaiveDate) -> Purchase {
        Purchase {
            member,
            money,
            is_food,
            date,
        }
    }

    #[test]
    fn test_calc_stats_empty() {
        let m = Vec::new();
        let p = Vec::new();
        let trs = Vec::new();
        let prs = Vec::new();

        let res = calc_stats(m.clone(), p, &trs, prs);
        assert_eq!(res, Vec::new());
    }

    #[test]
    fn test_calc_stats_single_member() {
        let id = Uuid::now_v7();
        let mut stat = MemberStat::new(id);

        let m = vec![id];
        let mut p = Vec::new();
        let trs = Vec::new();
        let mut prs = Vec::new();

        let res = calc_stats(m.clone(), p.clone(), &trs, prs.clone());
        assert_eq!(res, vec![stat]);

        let x = DateTime::UNIX_EPOCH;
        prs.push(Purchase {
            member: id,
            money: 100,
            is_food: true,
            date: x.date_naive(),
        });
        stat.food_purchases += 100;
        let res = calc_stats(m.clone(), p.clone(), &trs, prs.clone());
        assert_eq!(res, vec![stat]);

        p.push(Portion {
            member: id,
            date: x.date_naive(),
            cnt: 5,
        });
        stat.food_spent += 100;
        stat.portions += 5;
        let res = calc_stats(m.clone(), p.clone(), &trs, prs.clone());
        assert_eq!(res, vec![stat]);
    }

    #[test]
    fn test_calc_stats_multiple_members() {
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let members = vec![id1, id2];
        let portions = vec![
            create_portion(id1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 2),
            create_portion(id2, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 3),
        ];
        let trs = vec![
            create_transaction(id1, id2, 5),
            create_transaction(id2, id1, 3),
        ];
        let prs = vec![
            create_purchase(id1, 100, true, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            create_purchase(
                id2,
                200,
                false,
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ),
        ];

        let mut result = calc_stats(members, portions, &trs, prs);
        result.sort_by_key(|x| x.id);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, id1);
        assert_eq!(result[0].portions, 2);
        assert_eq!(result[0].food_purchases, 100);
        assert_eq!(result[0].not_food_purchases, 0);
        assert_eq!(result[0].transactions, 5 - 3);
        assert_eq!(result[0].food_spent, 40);
        assert_eq!(result[1].id, id2);
        assert_eq!(result[1].portions, 3);
        assert_eq!(result[1].food_purchases, 0);
        assert_eq!(result[1].not_food_purchases, 200);
        assert_eq!(result[1].transactions, 3 - 5);
        assert_eq!(result[1].food_spent, 60);
    }

    #[test]
    fn test_accounting_empty() {
        let members = vec![];

        let result = accounting(&members);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_accounting_single_member() {
        let id = Uuid::now_v7();
        let member_stat = MemberStat::new(id);

        let result = accounting(&vec![member_stat]);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_accounting_multiple_members() {
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let members = vec![id1, id2];
        let portions = vec![
            create_portion(id1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 2),
            create_portion(id2, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 3),
        ];
        let trs = vec![
            create_transaction(id1, id2, 5),
            create_transaction(id2, id1, 3),
        ];
        let prs = vec![
            create_purchase(id1, 100, true, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            create_purchase(
                id2,
                200,
                false,
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            ),
        ];

        let result = accounting(&calc_stats(members, portions, &trs, prs));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].member_from, id1);
        assert_eq!(result[0].member_to, id2);
        assert_eq!(result[0].money, 38);
        // id1: 100 - 20 * 2 + 5 - 3 = 62
        // id2: 200 - 20 * 3 - 5 + 3 = 138
    }

    #[test]
    fn test_calc_stats_multi_day() {
        let id = Uuid::now_v7();
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        let d3 = NaiveDate::from_ymd_opt(2026, 1, 3).unwrap();

        let portions = vec![
            create_portion(id, d1, 1),
            create_portion(id, d2, 1),
            create_portion(id, d3, 1),
        ];
        let prs = vec![
            create_purchase(id, 30, true, d1),
            create_purchase(id, 60, true, d2),
            create_purchase(id, 90, true, d3),
        ];

        let res = calc_stats(vec![id], portions, &[], prs);
        assert_eq!(res[0].food_spent, 135);
    }

    #[test]
    fn test_calc_stats_window_expires_old_purchases() {
        let id = Uuid::now_v7();
        let old = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let new = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(); // 45 дней

        let portions = vec![create_portion(id, old, 1), create_portion(id, new, 1)];
        let prs = vec![
            create_purchase(id, 100, true, old),
            create_purchase(id, 200, true, new),
        ];

        let res = calc_stats(vec![id], portions, &[], prs);
        // old: 100, new: только 200 в окне, 1×200 = 200. Итого 300.
        assert_eq!(res[0].food_spent, 300);
    }

    #[test]
    fn test_calc_stats_window_boundary_30_days() {
        let id = Uuid::now_v7();
        let d1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let d2 = d1 + Duration::days(30); // ровно 30 дней спустя

        let portions = vec![create_portion(id, d1, 1), create_portion(id, d2, 1)];
        let prs = vec![
            create_purchase(id, 100, true, d1),
            create_purchase(id, 100, true, d2),
        ];

        let res = calc_stats(vec![id], portions, &[], prs);
        // При окне [d2-29, d2] d1 выпадает. На d2: 1 порция, 100 трат → 100.
        // Итого: 100 (d1) + 100 (d2) = 200.
        assert_eq!(res[0].food_spent, 200);
    }

    #[test]
    fn test_calc_stats_multiple_portions_same_day() {
        let id = Uuid::now_v7();
        let d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let portions = vec![create_portion(id, d, 2), create_portion(id, d, 3)];
        let prs = vec![create_purchase(id, 100, true, d)];

        let res = calc_stats(vec![id], portions, &[], prs);
        assert_eq!(res[0].portions, 5);
        assert_eq!(res[0].food_spent, 100);
    }

    #[test]
    fn test_delta_ordering_by_value() {
        let negative_large = Delta {
            val: -100,
            id: Uuid::from_u128(2),
        };
        let positive_small = Delta {
            val: 100,
            id: Uuid::from_u128(1),
        };
        assert!(negative_large < positive_small);
    }
}
