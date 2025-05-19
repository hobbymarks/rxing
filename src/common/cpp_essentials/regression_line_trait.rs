use crate::common::Result;
use crate::{point, Point};

pub trait RegressionLineTrait {
    //     points: Vec<Point>,
    //     direction_inward: Point,

    // }
    // impl RegressionLine {
    // std::vector<PointF> _points;
    // PointF _directionInward;
    // PointF::value_t a = NAN, b = NAN, c = NAN;

    fn intersect<T: RegressionLineTrait, T2: RegressionLineTrait>(
        l1: &T,
        l2: &T2,
    ) -> Option<Point> {
        if !(l1.isValid() && l2.isValid()) {
            return None;
        }

        let d = l1.a() * l2.b() - l1.b() * l2.a();
        let x = (l1.c() * l2.b() - l1.b() * l2.c()) / d;
        let y = (l1.a() * l2.c() - l1.c() * l2.a()) / d;

        Some(point(x, y))
    }

    //  fn evaluate_begin_end(&self, begin: Point, end: Point) -> bool;// {
    // {
    // 	let mean = std::accumulate(begin, end, PointF()) / std::distance(begin, end);
    // 	PointF::value_t sumXX = 0, sumYY = 0, sumXY = 0;
    // 	for (auto p = begin; p != end; ++p) {
    // 		auto d = *p - mean;
    // 		sumXX += d.x * d.x;
    // 		sumYY += d.y * d.y;
    // 		sumXY += d.x * d.y;
    // 	}
    // 	if (sumYY >= sumXX) {
    // 		auto l = std::sqrt(sumYY * sumYY + sumXY * sumXY);
    // 		a = +sumYY / l;
    // 		b = -sumXY / l;
    // 	} else {
    // 		auto l = std::sqrt(sumXX * sumXX + sumXY * sumXY);
    // 		a = +sumXY / l;
    // 		b = -sumXX / l;
    // 	}
    // 	if (dot(_directionInward, normal()) < 0) {
    // 		a = -a;
    // 		b = -b;
    // 	}
    // 	c = dot(normal(), mean); // (a*mean.x + b*mean.y);
    // 	return dot(_directionInward, normal()) > 0.5f; // angle between original and new direction is at most 60 degree
    // }

    fn evaluate(&mut self, points: &[Point]) -> bool; // { return self.evaluate_begin_end(&points.front(), &points.back() + 1); }
    fn evaluateSelf(&mut self) -> bool;

    // RegressionLine() { _points.reserve(16); } // arbitrary but plausible start size (tiny performance improvement)

    // template<typename T> RegressionLine(PointT<T> a, PointT<T> b)
    // {
    // 	evaluate(std::vector{a, b});
    // }

    // template<typename T> RegressionLine(const PointT<T>* b, const PointT<T>* e)
    // {
    // 	evaluate(b, e);
    // }

    fn points(&self) -> &[Point]; //const { return _points; }
    fn length(&self) -> u32; //const { return _points.size() >= 2 ? int(distance(_points.front(), _points.back())) : 0; }
    fn isValid(&self) -> bool; //const { return !std::isnan(a); }
    fn normal(&self) -> Point; //const { return isValid() ? PointF(a, b) : _directionInward; }
    fn signedDistance(&self, p: Point) -> f32; //const { return dot(normal(), p) - c; }
    fn distance_single(&self, p: Point) -> f32; //const { return std::abs(signedDistance(PointF(p))); }
    fn project(&self, p: Point) -> Point {
        p - self.normal() * self.signedDistance(p)
    }

    fn reset(&mut self);
    // {
    // 	_points.clear();
    // 	_directionInward = {};
    // 	a = b = c = NAN;
    // }

    fn add(&mut self, p: Point) -> Result<()>; //{
                                               // 	assert(_directionInward != PointF());
                                               // 	_points.push_back(p);
                                               // 	if (_points.size() == 1)
                                               // 		c = dot(normal(), p);
                                               // }

    fn pop_back(&mut self); // { _points.pop_back(); }

    fn setDirectionInward(&mut self, d: Point); //{ _directionInward = normalized(d); }

    // fn evaluate(&self, double maxSignedDist = -1, bool updatePoints = false) -> bool
    fn evaluate_max_distance(
        &mut self,
        maxSignedDist: Option<f64>,
        updatePoints: Option<bool>,
    ) -> bool;
    // 	{
    // 		bool ret = evaluate(_points);
    // 		if (maxSignedDist > 0) {
    // 			auto points = _points;
    // 			while (true) {
    // 				auto old_points_size = points.size();
    // 				// remove points that are further 'inside' than maxSignedDist or further 'outside' than 2 x maxSignedDist
    // 				auto end = std::remove_if(points.begin(), points.end(), [this, maxSignedDist](auto p) {
    // 					auto sd = this->signedDistance(p);
    //                     return sd > maxSignedDist || sd < -2 * maxSignedDist;
    // 				});
    // 				points.erase(end, points.end());
    // 				if (old_points_size == points.size())
    // 					break;
    // // #ifdef PRINT_DEBUG
    // // 				printf("removed %zu points\n", old_points_size - points.size());
    // // #endif
    // 				ret = evaluate(points);
    // 			}

    // 			if (updatePoints)
    // 				_points = std::move(points);
    // 		}
    // 		return ret;
    // 	}

    fn isHighRes(&self) -> bool; //const
                                 // {
                                 // 	PointF min = _points.front(), max = _points.front();
                                 // 	for (auto p : _points) {
                                 // 		min.x = std::min(min.x, p.x);
                                 // 		min.y = std::min(min.y, p.y);
                                 // 		max.x = std::max(max.x, p.x);
                                 // 		max.y = std::max(max.y, p.y);
                                 // 	}
                                 // 	auto diff  = max - min;
                                 // 	auto len   = maxAbsComponent(diff);
                                 // 	auto steps = std::min(std::abs(diff.x), std::abs(diff.y));
                                 // 	// due to aliasing we get bad extrapolations if the line is short and too close to vertical/horizontal
                                 // 	return steps > 2 || len > 50;
                                 // }
    fn a(&self) -> f32;
    fn b(&self) -> f32;
    fn c(&self) -> f32;
}
